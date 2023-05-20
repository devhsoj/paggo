#[derive(Debug)]
pub enum Command {
    PING = 0,
    QUIT = 1,
    GET = 2,
    SET = 3,
    EXISTS = 4,
    DELETE = 5,
    UNKNOWN = 255
}

impl Command {
    pub fn from_u8(c: u8) -> Command {
        match c {
            0 => Command::PING,
            1 => Command::QUIT,
            2 => Command::GET,
            3 => Command::SET,
            4 => Command::EXISTS,
            5 => Command::DELETE,
            _ => Command::UNKNOWN,
        }
    }
}