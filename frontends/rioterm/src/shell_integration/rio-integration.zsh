# Reports the working directory to rio (OSC 7) on every prompt and
# directory change, which powers the title path variables and opening
# new tabs in the current directory. Loading twice is a no-op.
if [[ -n "${_rio_integration+x}" ]]; then
    return 0
fi
builtin typeset -g _rio_integration=1
builtin typeset -g _rio_last_pwd
builtin typeset -g _rio_wsl_distribution
if [[ -n "${RIO_WSL_INTEGRATION-}" ]]; then
    _rio_wsl_distribution=${WSL_DISTRO_NAME-}
    builtin unset RIO_WSL_INTEGRATION
fi

_rio_report_pwd() {
    [[ "${_rio_last_pwd-}" == "$PWD" ]] && return 0
    if [[ -n "$_rio_wsl_distribution" ]]; then
        # Windows Rio needs a local WSL share, including its distribution.
        # Parameter expansion replaces wslpath without forking a process.
        builtin printf '\e]9;9;\\\\wsl.localhost\\%s%s\a' \
            "$_rio_wsl_distribution" "${PWD//\//\\}"
    else
        builtin printf '\e]7;kitty-shell-cwd://%s%s\a' "${HOST-}" "$PWD"
    fi
    _rio_last_pwd=$PWD
}

builtin autoload -Uz add-zsh-hook
add-zsh-hook chpwd _rio_report_pwd
add-zsh-hook precmd _rio_report_pwd
_rio_report_pwd
