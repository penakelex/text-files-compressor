pub(crate) const HELP_TEMPLATE: &str = "\
{before-help}{about-with-newline}
Использование: {usage}

{all-args}{after-help}
";

pub(crate) const AFTER_HELP: &str = "\
\n\x1b[1m\x1b[4mФОРМАТ ВЫВОДА:\x1b[0m
    Каждый файл начинается с '### path'. Пустых строк между файлами нет.
    --indent-rle: 4 пробела отступа → один символ RLE ('\\t').
    --eol-mode:   auto — как в исходнике, lf — все '\\n', crlf — все '\\r\\n'.
    --collapse-blank-lines N: не более N подряд пустых строк (0 — убрать все).

\x1b[1m\x1b[4mФОРМАТ СПИСКОВ:\x1b[0m
    Списки передаются через запятую, пробел или повторением флага:
        --extensions kt,kts,xml
        --extensions kt kts xml
        --extensions kt --extensions kts
    INPUT_PATH должен идти первым аргументом после подкоманды:
        script compress ./project --extensions kt kts

\x1b[1m\x1b[4mПРИМЕРЫ:\x1b[0m
    script compress ./project --out full.txt \\
        --include-paths src/main,config/campaign.example.json \\
        --exclude-paths gradle/wrapper,build \\
        --rle --eol-mode lf --trim-trailing --collapse-blank-lines 1

    script git-diff ./project --out diff.txt \\
        --extensions java,kts,xml,json,toml,md,yml,yaml
";
