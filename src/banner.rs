// banner.rs
use colored::Colorize;

pub struct Banner {}

impl Banner {
    pub fn print_banner() {
        let banner_lines = vec![
            (
                vec![
                    "██╗  ██╗██╗   ██╗██████╗ ██████╗      ",
                    "██║  ██║╚██╗ ██╔╝██╔══██╗██╔══██╗     ",
                    "███████║ ╚████╔╝ ██║  ██║██████╔╝  ███",
                    "██╔══██║  ╚██╔╝  ██║  ██║██╔══██╗  ╚══",
                    "██║  ██║   ██║   ██████╔╝██║  ██║     ",
                    "╚═╝  ╚═╝   ╚═╝   ╚═════╝ ╚═╝  ╚═╝     ",
                ],
                "cyan",
            ),
            (
                vec![
                    "      ███████╗██╗  ██╗██╗██████╗ ███████╗",
                    "      ██╔══██║╚██╗██╔╝██║██╔══██╗██╔════╝",
                    "███╗  ██║  ██║ ╚███╔╝ ██║██║  ██║█████╗",
                    "═══╝  ██║  ██║ ██╔██╗ ██║██║  ██║██╔══╝",
                    "      ███████║██╔╝ ██╗██║██████╔╝███████║",
                    "      ╚══════╝╚═╝  ╚═╝╚═╝╚═════╝ ╚══════╝",
                ],
                "rust",
            ),
        ];

        for i in 0..6 {
            let hydr_line = &banner_lines[0].0[i];
            let oxide_line = &banner_lines[1].0[i];

            let colored_hydr = match banner_lines[0].1 {
                "cyan" => hydr_line.bright_cyan(),
                _ => hydr_line.normal(),
            };

            let colored_oxide = match banner_lines[1].1 {
                "rust" => oxide_line.truecolor(204, 85, 0),
                _ => oxide_line.normal(),
            };

            println!("{}{}", colored_hydr, colored_oxide);
        }
    }
}
