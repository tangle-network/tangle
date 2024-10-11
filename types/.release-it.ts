import type { Config } from 'release-it'

export default {
  hooks: {
    'after:bump': 'yarn run build',
  },
  git: {
    commitMessage: 'Release `tangle-substrate-types` v${version}',
    tagName: '${npm.name}/v${version}',
    tagAnnotation: 'Release ${npm.name} v${version}',
    pushRepo: 'git@github.com:tangle-network/tangle.git',
  },
  github: {
    release: false,
    releaseName: 'Release ${npm.name} v${version}',
    releaseNotes(context) {
      return `Release ${context.npm.name} version ${context.version}`
    },
  },
  npm: {
    publish: true,
  },
} satisfies Config
