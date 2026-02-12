import fs from "node:fs";
import prompts from "prompts";

(async () => {
  const response = await prompts({
    type: "text",
    name: 'packageName',
    message: 'What is the package name?',
    validate: value => value.length > 0 ? true : 'Package name is required'
  });

  if (!response.packageName) {
    console.error('Package name is required');
    process.exit(1);
  }

  // Update Cargo.toml
  const cargoToml = fs.readFileSync('Cargo.toml', 'utf8');
  const updatedCargoToml = cargoToml.replace(/name = "packageName"/g, `name = "${response.packageName}"`);
  fs.writeFileSync('Cargo.toml', updatedCargoToml);

  // Update README.md
  const readme = fs.readFileSync('README.md', 'utf8');
  const updatedReadme = readme.replace(/packageName/g, response.packageName);
  fs.writeFileSync('README.md', updatedReadme);
})();
