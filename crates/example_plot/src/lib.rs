use std::{fs, io, path::Path};

use plotters::prelude::{
    BitMapBackend, ChartBuilder, DrawingAreaErrorKind, IntoDrawingArea, IntoFont, RGBColor,
    Rectangle, Text, BLACK, WHITE,
};
use plotters::style::Color;
use thiserror::Error;

/// Errors that can occur while generating the demo plot image.
#[derive(Debug, Error)]
pub enum PlotGenerationError {
    /// Errors bubbling up from file system interactions.
    #[error("failed to prepare output directory: {0}")]
    Io(#[from] io::Error),
    /// Errors reported by the Plotters drawing backend.
    #[error("plotters drawing error: {0}")]
    Plot(String),
}

impl PlotGenerationError {
    fn from_plotters_error<E>(error: DrawingAreaErrorKind<E>) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Plot(format!("{error:?}"))
    }
}

/// Generates a simple PNG visualizing accessibility checklist progress.
///
/// The resulting chart is intended to document how asset generation can be
/// integrated into build scripts.
pub fn generate_accessibility_plot<P>(path: P) -> Result<(), PlotGenerationError>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let checkpoints: [(&str, i32); 4] = [
        ("Design tokens", 90),
        ("Focus order", 75),
        ("Keyboard maps", 80),
        ("Command palette", 95),
    ];

    let root = BitMapBackend::new(path, (640, 360)).into_drawing_area();
    root.fill(&WHITE)
        .map_err(PlotGenerationError::from_plotters_error)?;

    let x_range = 0..checkpoints.len() as i32;
    let mut chart = ChartBuilder::on(&root)
        .caption("Accessibility checklist coverage", ("sans-serif", 24))
        .margin(20)
        .x_label_area_size(60)
        .y_label_area_size(60)
        .build_cartesian_2d(x_range.clone(), 0..100)
        .map_err(PlotGenerationError::from_plotters_error)?;

    chart
        .configure_mesh()
        .disable_mesh()
        .x_desc("Checklist items")
        .y_desc("Completion (%)")
        .x_labels(checkpoints.len())
        .x_label_formatter(&|idx| {
            checkpoints
                .get(*idx as usize)
                .map(|(label, _)| (*label).to_string())
                .unwrap_or_default()
        })
        .y_label_formatter(&|value| format!("{value}%"))
        .y_labels(6)
        .draw()
        .map_err(PlotGenerationError::from_plotters_error)?;

    chart
        .draw_series(checkpoints.iter().enumerate().map(|(index, (_, value))| {
            let x0 = index as i32;
            Rectangle::new(
                [(x0, 0), (x0 + 1, *value)],
                RGBColor(37, 99, 235).mix(0.8).filled(),
            )
        }))
        .map_err(PlotGenerationError::from_plotters_error)?;

    chart
        .draw_series(checkpoints.iter().enumerate().map(|(index, (_, value))| {
            let label_position = (index as i32, value + 5);
            let text = format!("{value}%");
            Text::new(
                text,
                label_position,
                ("sans-serif", 16).into_font().color(&BLACK),
            )
        }))
        .map_err(PlotGenerationError::from_plotters_error)?;

    root.present()
        .map_err(PlotGenerationError::from_plotters_error)?;
    Ok(())
}
