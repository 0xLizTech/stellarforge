# Bash completion for stellarforge-nav-reporter. Regenerate with npm run generate-completions.
_stellarforge_nav_reporter() {
  local current="${COMP_WORDS[COMP_CWORD]}"
  local previous="${COMP_WORDS[COMP_CWORD-1]}"
  local options="--appraisal --oracle --appraiser-key --reporter-key-file --relayer-key-file --network --rpc-url --config --dry-run --no-dry-run --json --no-json --help"
  case "$previous" in
    --network) COMPREPLY=( $(compgen -W "testnet mainnet" -- "$current") ); return ;;
    --appraisal|--reporter-key-file|--relayer-key-file|--config) COMPREPLY=( $(compgen -f -- "$current") ); return ;;
    --oracle|--appraiser-key|--rpc-url) COMPREPLY=(); return ;;
  esac
  COMPREPLY=( $(compgen -W "$options" -- "$current") )
}
complete -F _stellarforge_nav_reporter stellarforge-nav-reporter
