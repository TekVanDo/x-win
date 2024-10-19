const path = require('node:path');
const fs = require('node:fs');

if (process.env.TAG) {
  const version = process.env.TAG.startsWith('v') ? process.env.TAG.substring(1) : process.env.TAG;

  console.log('[CLEANUP-PACKAGE]', 'Update package.json for', version);

  const packageJsonPath = path.join(process.cwd(), 'package.json');

  if (!fs.existsSync(packageJsonPath)) {
    console.error('package.json file not found!');
    process.exit(1);
  }

  const packageJson = require(packageJsonPath);
  console.log('[CLEANUP-PACKAGE]', 'Updating package.json...');

  // Clean up package.json for publish version to avoid installing none required deps for developers
  delete packageJson.devDependencies.ava;
  delete packageJson.devDependencies.husky;
  delete packageJson.ava;
  delete packageJson.packageManager;
  packageJson.scripts = {};

  fs.writeFileSync(packageJsonPath, JSON.stringify(packageJson, undefined, 2));

  console.log('[CLEANUP-PACKAGE]', 'package.json updated.');
  console.log('[CLEANUP-PACKAGE]', 'Finished.');
} else {
  console.error('TAG env not found!');
  process.exit(1);
}

/** Exist process with success */
process.exit(0);