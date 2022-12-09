use jstation_derive::ParamGroup;

#[derive(Debug, Default, ParamGroup)]
pub struct NoiseGate {
    #[boolean(param_nb = 16, cc_nb = 41, display_raw)]
    pub gate_on: GateOn,
    #[discrete(max = 10, param_nb = 17, cc_nb = 42, display_raw)]
    pub attack_time: AttackTime,
    #[discrete(min = 1, max = 99, param_nb = 18, cc_nb = 43, display_raw)]
    pub threshold: Threshold,
}
