use std::io::Write;


pub fn write<'a>(mut dst: impl Write, it: impl IntoIterator<Item=(&'a str, &'a str)>) -> std::io::Result<()> {
    for (k, v) in it {
        writeln!(dst, "%{k}%\n{v}\n")?;
    }
    Ok(())
}

