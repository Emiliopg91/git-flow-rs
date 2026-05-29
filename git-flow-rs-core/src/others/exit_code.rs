pub enum ExitCode {
    Ok = 0,
    NotAGitRepository = 1,
    UncommitedChanges = 2,
}
impl ExitCode {
    pub fn code(self) -> i32 {
        self as i32
    }
}
