use brocproc::{xprintb, xprintln};
use broccli::errors::Error;
use broccolor::Color;

fn main() -> Result<(), Error> {

    xprintb!(
        "test" => Color::Red,
        ("Hello {0} {1}" => Color::Magenta, "Fucking", "World" => Color::Blue),
        {
            "test" => Color::Green,
            {
                {
                    {
                        {
                            "Innnnnn" => Color::Magenta,
                        },
                        ("Test {1} {0}", "Heyyy", "Fuck")
                    }
                }
            }
        },
        "test",
    );

    /*
    let interface = Interface::new();

    interface.clear()?;

    if let Ok((cols, rows)) = get_terminal_size() {
        let config = BoxConfig {
            x: 0,
            y: 0,
            width: cols,
            height: rows,
            border_style: LineStyle::Single,
            color: Color::Transparent,
            background: Color::Transparent,
        };

        interface.draw_box(config)?;
    }*/

    Ok(())
}
