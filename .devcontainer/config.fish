set fish_greeting

set -x VISUAL vim
set -x EDITOR "$VISUAL"

alias clean 'find . -name Cargo.toml -type f -execdir cargo clean \;'
alias ll 'ls -lah --color=always'
alias ls 'ls --color'
alias fuck 'git reset --soft HEAD~1'

function allfilesize
    du -ah . | grep -v '/$' | sort -rh $argv
end

function fish_prompt
    # User in cyan
    set_color cyan; echo -n (whoami)

    # '@' in red
    set_color red; echo -n '@'

    # Hostname in green
    set_color green; echo -n 'arrakis'

    # ':' in purple
    set_color purple; echo -n ':'

    # Working directory in magenta
    set_color magenta; echo -n (prompt_pwd)

    # '$' in red
    set_color red; echo -n '$'

    # Reset color to default
    set_color normal

    # Print a space for separation
    echo -n ' '
end
