use models::verdicts::TotalVerdict;

pub trait MapLogExt<T, E: std::error::Error> {
    fn map_log(self, verdict: TotalVerdict) -> Result<T, TotalVerdict>;
}

impl<T, E: std::error::Error> MapLogExt<T, E> for Result<T, E> {
    fn map_log(self, verdict: TotalVerdict) -> Result<T, TotalVerdict> {
        match self {
            Ok(value) => Ok(value),
            Err(e) => {
                log::error!("{e}");
                Err(verdict)
            }
        }
    }
}
