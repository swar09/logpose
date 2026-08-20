use qrcode::{EcLevel, QrCode};
use qrcode::render::svg;

pub fn generate_qr_svg(
    content: &str,
    dark_color: Option<&str>,
    light_color: Option<&str>,
) -> Result<String, String> {
    let code = QrCode::with_error_correction_level(content.as_bytes(), EcLevel::H)
        .map_err(|e| format!("Failed to generate QR code: {}", e))?;

    let dark = dark_color.unwrap_or("#000000");
    let light = light_color.unwrap_or("#ffffff");

    let svg_string = code
        .render::<svg::Color>()
        .min_dimensions(300, 300)
        .dark_color(svg::Color(dark))
        .light_color(svg::Color(light))
        .build();

    Ok(svg_string)
}
