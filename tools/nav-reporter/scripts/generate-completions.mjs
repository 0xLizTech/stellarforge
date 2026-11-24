#!/usr/bin/env node
import { mkdirSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const outputIndex = process.argv.indexOf("--output-dir");
const outputDir = outputIndex >= 0
  ? path.resolve(process.argv[outputIndex + 1] ?? "")
  : path.resolve(here, "../completions");
if (outputIndex >= 0 && !process.argv[outputIndex + 1]) {
  throw new Error("--output-dir requires a directory");
}
mkdirSync(outputDir, { recursive: true });

const files = {
  "stellarforge-nav-reporter.bash": `# Bash completion for stellarforge-nav-reporter. Regenerate with npm run generate-completions.
_stellarforge_nav_reporter() {
  local current="\${COMP_WORDS[COMP_CWORD]}"
  local previous="\${COMP_WORDS[COMP_CWORD-1]}"
  local options="--appraisal --oracle --appraiser-key --reporter-key-file --relayer-key-file --network --rpc-url --config --dry-run --no-dry-run --json --no-json --help"
  case "$previous" in
    --network) COMPREPLY=( $(compgen -W "testnet mainnet" -- "$current") ); return ;;
    --appraisal|--reporter-key-file|--relayer-key-file|--config) COMPREPLY=( $(compgen -f -- "$current") ); return ;;
    --oracle|--appraiser-key|--rpc-url) COMPREPLY=(); return ;;
  esac
  COMPREPLY=( $(compgen -W "$options" -- "$current") )
}
complete -F _stellarforge_nav_reporter stellarforge-nav-reporter
`,
  "_stellarforge-nav-reporter": `#compdef stellarforge-nav-reporter
# Zsh completion for stellarforge-nav-reporter. Regenerate with npm run generate-completions.
_arguments \\
  '--appraisal[Signed appraisal JSON file]:file:_files' \\
  '--oracle[Oracle adapter contract ID]:contract ID:' \\
  '--appraiser-key[Expected appraiser public key]:public key:' \\
  '--reporter-key-file[Reporter secret key file]:file:_files' \\
  '--relayer-key-file[Relayer secret key file]:file:_files' \\
  '--network[Target Stellar network]:network:(testnet mainnet)' \\
  '--rpc-url[Custom Soroban RPC URL]:URL:' \\
  '--config[Config JSON file]:file:_files' \\
  '--dry-run[Validate without submitting]' \\
  '--no-dry-run[Disable dry-run mode]' \\
  '--json[Emit JSON output]' \\
  '--no-json[Disable JSON output]' \\
  '--help[Show help]'
`,
  "stellarforge-nav-reporter.fish": `# Fish completion for stellarforge-nav-reporter. Regenerate with npm run generate-completions.
complete -c stellarforge-nav-reporter -l appraisal -r -F -d 'Signed appraisal JSON file'
complete -c stellarforge-nav-reporter -l oracle -r -d 'Oracle adapter contract ID'
complete -c stellarforge-nav-reporter -l appraiser-key -r -d 'Expected appraiser public key'
complete -c stellarforge-nav-reporter -l reporter-key-file -r -F -d 'Reporter secret key file'
complete -c stellarforge-nav-reporter -l relayer-key-file -r -F -d 'Relayer secret key file'
complete -c stellarforge-nav-reporter -l network -r -a 'testnet mainnet' -d 'Target Stellar network'
complete -c stellarforge-nav-reporter -l rpc-url -r -d 'Custom Soroban RPC URL'
complete -c stellarforge-nav-reporter -l config -r -F -d 'Config JSON file'
complete -c stellarforge-nav-reporter -l dry-run -d 'Validate without submitting'
complete -c stellarforge-nav-reporter -l no-dry-run -d 'Disable dry-run mode'
complete -c stellarforge-nav-reporter -l json -d 'Emit JSON output'
complete -c stellarforge-nav-reporter -l no-json -d 'Disable JSON output'
complete -c stellarforge-nav-reporter -l help -s h -d 'Show help'
`,
};
for (const [name, content] of Object.entries(files)) writeFileSync(path.join(outputDir, name), content);
process.stdout.write(`Generated ${Object.keys(files).length} completion files in ${outputDir}\n`);
