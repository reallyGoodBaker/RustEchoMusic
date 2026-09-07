use crate::errors::AppError;

pub trait AudioEngine: Send {
    fn play(&mut self) -> Result<(), AppError>;
    fn pause(&mut self);
    fn seek(&mut self, time: f64);
    fn set_volume(&self, volume: f32);
    fn set_pan(&self, pan: f32);
    fn current_time(&self) -> f64;
    fn paused(&self) -> bool;

    // ===== 均衡器（EQ）扩展方法 =====
    // 以下方法均带默认 no-op 实现，保证现有实现 / mock 不被破坏。

    fn set_eq_band_gain(&mut self, _band_index: usize, _gain_db: f64) -> Result<(), AppError> {
        Ok(())
    }

    fn apply_eq_preset(&mut self, _gains: &[f64; 10]) -> Result<(), AppError> {
        Ok(())
    }

    fn set_eq_enabled(&mut self, _enabled: bool) -> Result<(), AppError> {
        Ok(())
    }

    fn get_eq_bands(&self) -> Result<[f64; 10], AppError> {
        Ok([0.0; 10])
    }

    fn is_eq_enabled(&self) -> Result<bool, AppError> {
        Ok(false)
    }
}
