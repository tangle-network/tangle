#!/usr/bin/env node
// Copyright 2017-2020 @polkadot/dev authors & contributors
// SPDX-License-Identifier: Apache-2.0

const babel = require('@babel/cli/lib/babel/dir').default;
const path = require('path');
const mkdirp = require('mkdirp');
const { execSync } = require('child_process');
const fs = require('fs-extra');
const glob = require('glob');
const glob2base = require('glob2base');
const { Minimatch } = require('minimatch');

function normalizePath(originalPath) {
  const normalizedPath = path.relative(process.cwd(), path.resolve(originalPath)).replace(/\\/g, '/');

  return /\/$/.test(normalizedPath) ? normalizedPath.slice(0, -1) : normalizedPath || '.';
}

const copySync = (src, dst) => {
  const normalizedSource = normalizePath(src);
  const normalizedOutputDir = normalizePath(dst);
  const baseDir = normalizePath(glob2base({ minimatch: new Minimatch(normalizedSource) }));

  glob
    .sync(normalizedSource, {
      follow: false,
      nodir: true,
      silent: true
    })
    .forEach((src) => {
      const dst = baseDir === '.' ? path.join(normalizedOutputDir, src) : src.replace(baseDir, normalizedOutputDir);

      if (dst !== src) {
        const stat = fs.statSync(src);

        if (stat.isDirectory()) {
          fs.ensureDirSync(dst);
        } else {
          fs.ensureDirSync(path.dirname(dst));
          fs.copySync(src, dst);
        }

        fs.chmodSync(dst, stat.mode);
      }
    });
}
const executeSync = (cmd) => {
  try {
    execSync(cmd, { stdio: 'inherit' });
  } catch (error) {
    process.exit(-1);
  }
}

const CPX = ['css', 'gif', 'hbs', 'jpg', 'js', 'png', 'svg', 'd.ts']
  .map((ext) => `src/**/*.${ext}`)

function copyMiscFiles(dir) {
  // take the package.json defined in the module and strip out the '/build/'
  // in the export paths before copying to the output 'build' folder for publishing.
  // The 'build/' in the path exists for packages to resolve properly in builds.
  //    e.g. the '@webb-tools/test-utils' package needs this to properly resolve
  //         methods in '@webb-tools/sdk-core' because it imports '@webb-tools/utils'
  //         which uses '@webb-tools/sdk-core' in its dependencies. 
  const raw = fs.readFileSync('./package.json');
  const pkg = JSON.parse(raw);
  const pkgString = JSON.stringify(pkg);
  const newPkgString = pkgString.replaceAll('/build/', '/');

  // Extra logic for copying over other files (package.json for cjs modules)
  [...CPX]
    .concat(`./ts-types/src/**/*.d.ts`)
    .forEach((src) => copySync(src, 'build'));

  fs.writeFileSync('./build/package.json', newPkgString);
}

// @param module - the module system to use cjs or esm.
async function buildBabel(dir, module = 'esm') {
  console.log('build babel for: ', module);

  // babel configuratiom
  const configFileName = `babel-config-cjs.cjs`;

  // Prefer to use local config over the root one.
  const conf = path.join(process.cwd(), configFileName);

  // Commonjs builds will exist in a '/cjs' directory for the package.
  await babel({
    babelOptions: {
      configFile: conf
    },
    cliOptions: {
      extensions: ['.ts'],
      filenames: ['src'],
      ignore: '**/*.d.ts',
      outDir: path.join(process.cwd(), 'build'),
      outFileExtension: '.js'
    }
  });

  copyMiscFiles(dir, module);
}


async function buildJs(dir) {
  if (!fs.existsSync(path.join(process.cwd(), '.skip-build'))) {
    const { name, version } = require(path.join(process.cwd(), './package.json'));

    console.log(`*** ${name} ${version}`);

    mkdirp.sync('build');

    await buildBabel(dir, 'cjs');
    
    // copyMiscFiles();
  }
}

async function buildMonorepo() {
  executeSync('yarn clean');
  executeSync('tsc --emitDeclarationOnly --outdir ./ts-types');
  await buildJs('types');
}

async function main() {
  buildMonorepo();
}

main().catch((error) => {
  console.error(error);
  process.exit(-1);
});
