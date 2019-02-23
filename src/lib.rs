use std::io::{Result, Write};

enum State {
    Newline,
    Inline,
}

pub struct BraceWriter<W: Write> {
    inner: Option<W>,
    state: State,
    indent: usize,
}

impl<W: Write> BraceWriter<W> {
    pub fn new(inner: W) -> BraceWriter<W> {
        BraceWriter {
            inner: Some(inner),
            state: State::Newline,
            indent: 0,
        }
    }
}

fn indent(out: &mut Vec<u8>, depth: usize) {
    for _ in 0..depth {
        out.push('\t' as u8);
    }
}

impl<W: Write> Write for BraceWriter<W> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let mut out = Vec::with_capacity(buf.len());

        let mut iter = buf.iter().peekable();
        while let Some(cref) = iter.peek() {
            let c = **cref as char;
            self.state = match self.state {
                State::Newline => {
                    if c == '{' {
                        indent(&mut out, self.indent);
                        out.write(b"{\n")?;
                        self.indent += 1;
                        iter.next();
                        State::Newline
                    } else if c == '}' {
                        self.indent -= 1;
                        indent(&mut out, self.indent);
                        out.write(b"}\n")?;
                        iter.next();
                        State::Newline
                    } else if c.is_whitespace() {
                        iter.next();
                        State::Newline
                    } else {
                        indent(&mut out, self.indent);
                        State::Inline
                    }
                }
                State::Inline => {
                    if c == '\n' || c == '\r' {
                        out.write(b"\n")?;
                        iter.next();
                        State::Newline
                    } else if c == '{' || c == '}' {
                        out.write(b"\n")?;
                        State::Newline
                    } else {
                        out.push(c as u8);
                        iter.next();
                        State::Inline
                    }
                }
            }
        }

        self.inner.as_mut().unwrap().write(out.as_slice())?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {
        self.inner.as_mut().unwrap().flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use std::io::{Result, Write};

    #[test]
    fn basic_output() -> Result<()> {
        let mut writer = BraceWriter::new(io::stdout());
        writeln!(
            &mut writer,
            r#"
        if (apple == 0)
        {{
            
            whatever;
            if (i == 1) {{ printf( "hello"); }} }}

            "#
        )
    }
}
