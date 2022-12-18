use jstation_derive::ParameterGroup;

#[derive(Clone, Copy, Debug, Default, ParameterGroup)]
pub struct NoiseGate {
    #[boolean(param_nb = 16, cc_nb = 41)]
    pub switch: Switch,
    #[const_range(max = 10, param_nb = 17, cc_nb = 42, display_raw)]
    pub attack_time: AttackTime,
    #[const_range(min = 1, max = 99, param_nb = 18, cc_nb = 43, display_raw)]
    pub threshold: Threshold,
}
