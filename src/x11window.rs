//! Facilities to work with X11 windows command line tools

use crate::prelude::*;
use crate::shutils::{cmd, pipe};
use std::fs::File;

/// Retrieve the pixel dump of a given X window then write the content to a .pnm file.
fn write_window_pixels(wid: u64, pnm_out: &str) -> Result<()> {
    let file_out = File::create(pnm_out).unwrap();
    let piped = Stdio::from(file_out);

    let mut xwd_cmd = cmd(&["xwdtopnm"]);
    xwd_cmd.stdout(piped);

    let mut cmds = [
        &mut cmd(&["xwd", "-id", &format!("0x{:x}", wid)]),
        &mut xwd_cmd,
    ];

    // Execute the above commands piping them together
    let _ = pipe(&mut cmds);

    Ok(())
}

fn read_window_pixels(pnm_file: &str) -> Result<image::DynamicImage> {
    Ok(image::ImageReader::open(pnm_file)?.decode()?)
}

fn get_window_image(wid: u64) -> Result<image::DynamicImage> {
    let tmp_file = format!("{}.ppm", wid);

    let _ = write_window_pixels(wid, &tmp_file)?;
    let dyn_image = read_window_pixels(&tmp_file)?;

    // Now remove the temp file
    std::fs::remove_file(tmp_file)?;
    Ok(dyn_image)
}
