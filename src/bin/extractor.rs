use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::PathBuf;

use clap::Parser;
use itchy::MessageStream;

const STOCK_DIRECTORY: u8 = b'R';

/// Copy only the order-related ITCH messages for a given symbol into a new file.
/// First pass: find stock_locate for symbol via StockDirectory messages.
/// Second pass: copy raw frames verbatim if they match that stock_locate.
#[derive(Parser, Debug)]
struct Args {
    /// Input ITCH file (uncompressed)
    #[arg(short, long)]
    input: PathBuf,
    /// Output ITCH file (will be created/overwritten)
    #[arg(short, long)]
    output: PathBuf,
    /// Symbol to filter (case-insensitive)
    #[arg(short, long)]
    symbol: String,

    #[arg(long)]
    max_messages: Option<usize>,
}

fn is_order_tag(tag: u8) -> bool {
    matches!(
        tag,
        b'A' | // Add Order
        b'F' | // Add Order (Attributed)
        b'E' | // Order Executed
        b'C' | // Order Executed With Price
        b'X' | // Order Cancel
        b'D' | // Order Delete
        b'U' // Order Replace
    )
}

fn read_exact_or_eof<R: Read>(r: &mut R, buf: &mut [u8]) -> io::Result<bool> {
    let mut read = 0;
    while read < buf.len() {
        match r.read(&mut buf[read..])? {
            0 if read == 0 => return Ok(false), // clean EOF before any bytes
            0 => {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "truncated length",
                ))
            }
            n => read += n,
        }
    }
    Ok(true)
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    let symbol = args.symbol.to_lowercase();

    // --- Pass 1: discover stock_locate for symbol ---
    let mut locate: Option<u16> = None;
    let stream = MessageStream::from_file(&args.input).map_err(std::io::Error::other)?;

    let mut directory_started = false;

    for msg in stream {
        let msg = match msg {
            Ok(m) => m,
            Err(e) => {
                eprintln!("error decoding message: {e}");
                continue;
            }
        };
        if msg.tag != STOCK_DIRECTORY && directory_started {
            break; // directory section is finished
        }
        let itchy::Body::StockDirectory(dir) = &msg.body else {
            continue;
        };

        directory_started = true;

        if dir.stock.trim_end().to_lowercase() == symbol {
            locate = Some(msg.stock_locate);
            break;
        }
    }

    let Some(locate) = locate else {
        eprintln!("symbol {symbol} not found in stock directory");
        return Ok(());
    };
    eprintln!("found symbol {symbol} with stock_locate={locate}");

    // --- Pass 2: filter raw frames ---
    let infile = File::open(&args.input)?;
    let mut r = BufReader::with_capacity(1 << 20, infile);

    let outfile = File::create(&args.output)?;
    let mut w = BufWriter::with_capacity(1 << 20, outfile);

    let mut kept = 0usize;
    let mut total = 0usize;

    loop {
        if let Some(max) = args.max_messages {
            if kept >= max {
                break;
            }
        }

        // Frame: u16 length + payload
        let mut len_buf = [0u8; 2];
        if !read_exact_or_eof(&mut r, &mut len_buf)? {
            break; // EOF
        }
        let msg_len = u16::from_be_bytes(len_buf) as usize;
        let mut payload = vec![0u8; msg_len];
        r.read_exact(&mut payload)?;

        total += 1;

        if msg_len < 3 {
            continue; // malformed
        }

        let tag = payload[0];
        let stock_locate = u16::from_be_bytes([payload[1], payload[2]]);
        let keep = stock_locate == locate && is_order_tag(tag);

        if keep {
            w.write_all(&len_buf)?;
            w.write_all(&payload)?;
            kept += 1;
        }
    }

    w.flush()?;
    eprintln!(
        "Frames scanned: {total}, kept: {kept}{}",
        if let Some(max) = args.max_messages {
            format!(" (max_messages={max})")
        } else {
            "".to_string()
        }
    );
    Ok(())
}
