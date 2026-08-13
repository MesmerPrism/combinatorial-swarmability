/// Small deterministic generator used only for synthetic scene initialization.
pub(crate) struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    pub(crate) const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut value = self.state;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        value ^ (value >> 31)
    }

    pub(crate) fn signed_unit_f32(&mut self) -> f32 {
        let bits = u16::try_from(self.next_u64() >> 48).unwrap_or_default();
        let unit = f32::from(bits) / f32::from(u16::MAX);
        unit.mul_add(2.0, -1.0)
    }
}
