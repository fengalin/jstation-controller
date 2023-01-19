pub mod data;
pub use data::{dsp, CCParameter, Program, ProgramData, ProgramId, ProgramNb, ProgramsBank};

mod error;
pub use error::Error;

mod interface;
pub use interface::{Interface, Listener};

mod midi;
pub use midi::*;

pub mod procedure;
pub use procedure::{Procedure, ProcedureBuilder};

use std::{collections::BTreeMap, sync::Arc};

pub struct JStation {
    iface: Interface,
    dsp: dsp::Dsp,
    bank: ProgramsBank,
    programs: BTreeMap<ProgramId, Program>,
    cur_prog_id: Option<ProgramId>,
    has_changed: bool,
}

impl JStation {
    pub fn new(app_name: Arc<str>) -> Self {
        JStation {
            iface: Interface::new(app_name),
            dsp: dsp::Dsp::default(),
            bank: ProgramsBank::default(),
            programs: BTreeMap::new(),
            cur_prog_id: None,
            has_changed: false,
        }
    }

    pub fn clear(&mut self) {
        self.iface.clear();
        self.bank = ProgramsBank::default();
        self.programs.clear();
        self.cur_prog_id = None;
        self.has_changed = false;
    }

    pub fn iface(&self) -> &Interface {
        &self.iface
    }

    pub fn iface_mut(&mut self) -> &mut Interface {
        &mut self.iface
    }

    pub fn dsp(&self) -> &dsp::Dsp {
        &self.dsp
    }

    pub fn cur_prog_id(&self) -> Option<ProgramId> {
        self.cur_prog_id
    }

    pub fn programs_bank(&self) -> ProgramsBank {
        self.bank
    }

    pub fn get_program(&self, prog_id: ProgramId) -> Option<&Program> {
        self.programs.get(&prog_id)
    }

    pub fn has_changed(&self) -> bool {
        self.has_changed
    }

    pub fn handle_device(&mut self, msg: Message) -> Result<(), Error> {
        use Message::*;
        match msg {
            SysEx(sysex) => {
                use data::ProgramParameter;
                use Procedure::*;
                match Arc::try_unwrap(sysex).unwrap().proc {
                    NotifyStore(resp) => {
                        let prog_id = ProgramId::new_user(resp.nb);
                        self.cur_prog_id = Some(prog_id);
                        self.iface.request_program(prog_id).expect("Not connected");
                        self.has_changed = false;
                    }
                    NotifyUtility(_) => {
                        self.iface
                            .request_utility_settings()
                            .expect("Not connected");
                    }
                    WhoAmIResp(resp) => {
                        self.programs.clear();

                        self.iface.have_who_am_i_resp(resp).map_err(|err| {
                            self.clear();

                            err
                        })?;
                    }
                    UtilitySettingsResp(resp) => {
                        self.dsp.utility_settings = resp.try_into()?;

                        if self.programs.is_empty() {
                            self.iface.bank_dump()?;
                        }
                    }
                    ProgramIndicesResp(_) => (),
                    OneProgramResp(resp) => {
                        if self.cur_prog_id.is_some() {
                            self.dsp.set_from(resp.prog.data())?;
                        }

                        self.programs.insert(resp.prog.id(), resp.prog);
                    }
                    ProgramUpdateResp(resp) => {
                        self.dsp.set_from(&resp.prog_data)?;
                        self.has_changed = resp.has_changed;

                        let prog = self
                            .programs
                            .iter()
                            .find(|(_, prog)| !self.dsp.has_changed(prog.data()));

                        if let Some((_, prog)) = prog {
                            self.cur_prog_id = Some(prog.id());
                        } else {
                            // This can occur on startup when the program on device `has_changed`
                            // or if a factory program is selected.
                            self.cur_prog_id = None;
                            return Err(Error::ProgramIdenticationFailure);
                        }
                    }
                    StartBankDumpResp(_) => {
                        self.bank = ProgramsBank::default();
                    }
                    EndBankDumpResp(_) => {
                        self.iface.program_update_req()?;
                    }
                    ToMessageResp(resp) => match resp.res {
                        Ok(req_proc) => log::debug!("Proc. x{req_proc:02x}: Ok"),
                        Err(err) => panic!("{err}"),
                    },
                    other => {
                        log::debug!("Unhandled {other:?}");
                    }
                }
            }
            ChannelVoice(cv) => {
                use channel_voice::Message::*;
                use data::CCParameterSetter;
                match cv.msg {
                    CC(cc) => match self.dsp.set_cc(cc) {
                        Ok(Some(_)) => self.update_has_changed(),
                        Ok(None) => (),
                        Err(err) => log::warn!("{err}"),
                    },
                    ProgramChange(prog_id) => {
                        self.cur_prog_id = Some(prog_id);
                        self.bank = prog_id.bank();

                        self.load_prog(prog_id)?;
                    }
                }
            }
        }

        Ok(())
    }

    pub fn change_program(&mut self, id: ProgramId) -> Result<(), Error> {
        self.iface.change_program(id)?;

        self.cur_prog_id = Some(id);
        self.has_changed = false;

        self.load_prog(id)?;

        Ok(())
    }

    pub fn store_to(&mut self, nb: ProgramNb) -> Result<(), Error> {
        use data::ProgramParameter;

        let prog_id = ProgramId::new_user(nb);

        if self
            .cur_prog_id
            .map_or(true, |cur_prog_id| cur_prog_id != prog_id)
        {
            self.iface
                .change_program(prog_id)
                .expect("Changing to known user program");
            self.bank = ProgramsBank::User;
            self.cur_prog_id = Some(prog_id);
        }

        let prog = self
            .programs
            .get_mut(&prog_id)
            .expect("Storing to selected and known program");

        if self.dsp.has_changed(prog.data()) {
            self.dsp.store(prog.data_mut());

            // Technically, the program is actually stored on
            // device after reception of the Ok ack for proc x02.
            self.iface.store_program(prog)?;

            self.has_changed = false;
        }

        Ok(())
    }

    pub fn undo(&mut self) -> Result<(), Error> {
        self.iface.reload_program()?;
        if let Some(cur_prog_id) = self.cur_prog_id {
            self.load_prog(cur_prog_id)?;
        }

        self.has_changed = false;

        Ok(())
    }

    pub fn rename(&mut self, name: impl ToString) {
        self.dsp.name = ProgramData::format_name(name.to_string());
        self.update_has_changed();
    }

    pub fn select_bank(&mut self, bank: ProgramsBank) {
        self.bank = bank;
    }

    pub fn update_param(&mut self, param: dsp::Parameter) {
        use data::ParameterSetter;

        if self.dsp.set(param).is_some() {
            if let Some(cc) = param.to_cc() {
                // FIXME handle the error
                let _ = self.iface.send_cc(cc);
            } else {
                log::error!("No CC for {:?}", param);
            }

            self.update_has_changed();
        }
    }

    pub fn update_utility_settings(&mut self, settings: dsp::UtilitySettings) {
        self.dsp.utility_settings = settings;
        // FIXME send message to device
    }

    fn load_prog(&mut self, prog_id: ProgramId) -> Result<(), Error> {
        use data::ProgramParameter;

        if let Some(prog) = self.programs.get(&prog_id) {
            self.dsp.set_from(prog.data()).unwrap();
        } else if let Err(err) = self.iface.request_program(prog_id) {
            self.cur_prog_id = None;

            return Err(err);
        }

        Ok(())
    }

    fn update_has_changed(&mut self) {
        use data::ProgramParameter;

        let cur_prog = self
            .cur_prog_id
            .and_then(|prog_id| self.programs.get(&prog_id));
        if let Some(cur_prog) = cur_prog {
            self.has_changed = self.dsp.has_changed(cur_prog.data());
        }
    }
}
