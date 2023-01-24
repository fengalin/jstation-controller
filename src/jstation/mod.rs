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

pub mod prelude {
    pub use super::data::{
        BoolParameter, CCParameterSetter, ConstRangeParameter, DiscreteParameter, ParameterSetter,
        ProgramParameter, VariableRangeParameter,
    };
    pub use super::JStationImpl;
}

use std::{collections::BTreeMap, sync::Arc};

use prelude::*;

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

    fn load_prog(&mut self, prog_id: ProgramId) -> Result<(), Error> {
        if let Some(prog) = self.programs.get(&prog_id) {
            self.dsp.set_from(prog.data()).unwrap();
        } else if let Err(err) = self.iface.request_program(prog_id) {
            self.cur_prog_id = None;

            return Err(err);
        }

        Ok(())
    }

    fn update_has_changed(&mut self) {
        let cur_prog = self
            .cur_prog_id
            .and_then(|prog_id| self.programs.get(&prog_id));
        if let Some(cur_prog) = cur_prog {
            self.has_changed = self.dsp.has_changed(cur_prog.data());
        }
    }
}

impl JStationImpl for JStation {
    type Inner = Self;

    fn inner(&self) -> &Self::Inner {
        self
    }

    fn inner_mut(&mut self) -> &mut Self::Inner {
        self
    }

    fn iface(&self) -> &Interface {
        &self.iface
    }

    fn iface_mut(&mut self) -> &mut Interface {
        &mut self.iface
    }

    fn dsp(&self) -> &dsp::Dsp {
        &self.dsp
    }

    fn cur_prog_id(&self) -> Option<ProgramId> {
        self.cur_prog_id
    }

    fn programs_bank(&self) -> ProgramsBank {
        self.bank
    }

    fn get_program(&self, prog_id: ProgramId) -> Option<&Program> {
        self.programs.get(&prog_id)
    }

    fn has_changed(&self) -> bool {
        self.has_changed
    }

    fn clear(&mut self) {
        self.iface.clear();
        self.bank = ProgramsBank::default();
        self.programs.clear();
        self.cur_prog_id = None;
        self.has_changed = false;
    }

    fn handle_device(&mut self, msg: Message) -> Result<(), Error> {
        use Message::*;
        match msg {
            SysEx(sysex) => {
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
                        // When changing channel from the J-Station,
                        // `NotifyUtility` is sent on the new channel.
                        // Better change the channel from the application instead.
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
                match cv.msg {
                    CC(cc) => match self.dsp.set_cc(cc) {
                        Ok(Some(_)) => self.update_has_changed(),
                        Ok(None) => log::debug!("Unhandled {cc:?}"),
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

    fn change_program(&mut self, id: ProgramId) -> Result<(), Error> {
        self.iface.change_program(id)?;

        self.cur_prog_id = Some(id);
        self.has_changed = false;

        self.load_prog(id)?;

        Ok(())
    }

    fn store_to(&mut self, nb: ProgramNb) -> Result<(), Error> {
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

    fn undo(&mut self) -> Result<(), Error> {
        self.iface.reload_program()?;
        if let Some(cur_prog_id) = self.cur_prog_id {
            self.load_prog(cur_prog_id)?;
        }

        self.has_changed = false;

        Ok(())
    }

    fn rename(&mut self, name: impl ToString) {
        self.dsp.name = ProgramData::format_name(name.to_string());
        self.update_has_changed();
    }

    fn select_bank(&mut self, bank: ProgramsBank) {
        self.bank = bank;
    }

    fn update_param(&mut self, param: dsp::Parameter) {
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

    fn update_utility_settings(&mut self, settings: dsp::UtilitySettings) {
        if let Err(err) = self.iface.update_utility_settings(settings) {
            log::debug!("Failed to update device utility settings: {err}");
        }

        if self.dsp.utility_settings.midi_channel != settings.midi_channel {
            self.iface.change_chan(settings.midi_channel.into())
        }

        self.dsp.utility_settings = settings;
    }
}

/// `JStationImpl` common interface.
///
/// This trait can be implemented by decorators so they get
/// access to base implementation without the need to redefine
/// every methods.
pub trait JStationImpl {
    type Inner: JStationImpl;

    fn inner(&self) -> &Self::Inner;

    fn inner_mut(&mut self) -> &mut Self::Inner;

    fn iface(&self) -> &Interface {
        self.inner().iface()
    }

    fn iface_mut(&mut self) -> &mut Interface {
        self.inner_mut().iface_mut()
    }

    fn dsp(&self) -> &dsp::Dsp {
        self.inner().dsp()
    }

    fn cur_prog_id(&self) -> Option<ProgramId> {
        self.inner().cur_prog_id()
    }

    fn programs_bank(&self) -> ProgramsBank {
        self.inner().programs_bank()
    }

    fn get_program(&self, prog_id: ProgramId) -> Option<&Program> {
        self.inner().get_program(prog_id)
    }

    fn has_changed(&self) -> bool {
        self.inner().has_changed()
    }

    fn refresh(&mut self) -> Result<(), Error> {
        self.iface_mut().refresh()
    }

    fn clear(&mut self) {
        self.inner_mut().clear();
    }

    fn tuner_on(&mut self) -> Result<(), Error> {
        self.iface_mut().tuner_on()
    }

    fn tuner_off(&mut self) -> Result<(), Error> {
        self.iface_mut().tuner_off()
    }

    fn handle_device(&mut self, msg: Message) -> Result<(), Error> {
        self.inner_mut().handle_device(msg)
    }

    fn change_program(&mut self, id: ProgramId) -> Result<(), Error> {
        self.inner_mut().change_program(id)
    }

    fn store_to(&mut self, nb: ProgramNb) -> Result<(), Error> {
        self.inner_mut().store_to(nb)
    }

    fn undo(&mut self) -> Result<(), Error> {
        self.inner_mut().undo()
    }

    fn rename(&mut self, name: impl ToString) {
        self.inner_mut().rename(name);
    }

    fn select_bank(&mut self, bank: ProgramsBank) {
        self.inner_mut().select_bank(bank);
    }

    fn update_param(&mut self, param: dsp::Parameter) {
        self.inner_mut().update_param(param);
    }

    fn update_utility_settings(&mut self, settings: dsp::UtilitySettings) {
        self.inner_mut().update_utility_settings(settings);
    }
}
