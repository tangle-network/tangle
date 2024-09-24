import type { Config } from 'release-it'

export default {
  hooks: {
    'after:bump': 'yarn run build',
  },
  git: {
    commitMessage: 'Release `tangle-substrate-types` v${version}',
    tag: false,
    push: false,
  },
  github: {
    release: false,
  },
  npm: {
    publish: true,
  },
} satisfies Config
