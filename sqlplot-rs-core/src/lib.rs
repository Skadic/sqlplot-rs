use std::io::Write;

pub trait ResultLine {
    fn to_result_line(&self) -> String;

    fn write_result_line(&self, out: &mut impl Write) -> std::io::Result<()> {
        out.write_all(self.to_result_line().as_bytes())
    }
}
