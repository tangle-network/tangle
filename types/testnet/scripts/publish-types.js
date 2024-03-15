
const path = require('path');

const rimraf = require('rimraf');
const { execSync: _execSync } = require('child_process');

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

const execSync = (cmd) => _execSync(cmd, { stdio: 'inherit' });

function npmGetVersion() {
  return JSON.parse(fs.readFileSync(path.resolve(process.cwd(), 'package.json'), 'utf8')).version;
}

function npmPublish() {
  rimraf.sync('build/package.json');

  // take the package.json defined in the module and strip out the '/build/'
  // in the export paths before copying to the output 'build' folder for publishing.
  // The 'build/' in the path exists for packages to resolve properly while using
  // webb.js locally.
  //    e.g. the '@webb-tools/test-utils' package needs this to properly resolve
  //         methods in '@webb-tools/sdk-core' because it imports '@webb-tools/utils'
  //         which uses '@webb-tools/sdk-core' in its dependencies. 
  const raw = fs.readFileSync('./package.json');
  const pkg = JSON.parse(raw);
  const pkgString = JSON.stringify(pkg);
  const newPkgString = pkgString.replaceAll('/build/', '/');

  ['LICENSE', 'README.md'].forEach((file) => copySync(file, 'build'));
  fs.writeFileSync('./build/package.json', newPkgString);
  process.chdir('build');
  const tag = npmGetVersion().includes('-') ? '--tag beta' : '';
  let count = 1;

  while (true) {
    try {
      execSync(`npm publish --access public ${tag}`);

      break;
    } catch (error) {
      console.error(error);
      if (count < 5) {
        const end = Date.now() + 15000;

        console.error(`Publish failed on attempt ${count}/5. Retrying in 15s`);
        count++;

        while (Date.now() < end) {
          // just spin our wheels
        }
      }
    }
  }
}

npmPublish();
