import Ajv from 'ajv';
import fs from 'fs';
import path from 'path';

const ajv = new Ajv({ allErrors: true });

const configDir = 'config';
const schemaFile = path.join(configDir, 'schema.json');
const schema = JSON.parse(fs.readFileSync(schemaFile, 'utf8'));
const validate = ajv.compile(schema);

const files = fs.readdirSync(configDir);

function isObject(item) {
  return (item && typeof item === 'object' && !Array.isArray(item));
}

function deepMerge(target, source) {
  let output = { ...target };
  if (isObject(target) && isObject(source)) {
    Object.keys(source).forEach(key => {
      if (isObject(source[key])) {
        if (!(key in target))
          Object.assign(output, { [key]: source[key] });
        else
          output[key] = deepMerge(target[key], source[key]);
      } else {
        Object.assign(output, { [key]: source[key] });
      }
    });
  }
  return output;
}

function mergeConfig(config) {
  if (config.extends) {
    const parentFile = path.join(configDir, config.extends);
    if (fs.existsSync(parentFile)) {
      const parentConfig = JSON.parse(fs.readFileSync(parentFile, 'utf8'));
      // Recursively merge parent config
      const mergedParent = mergeConfig(parentConfig);
      // Merge current config over parent
      return deepMerge(mergedParent, config);
    }
  }
  return config;
}

let hasErrors = false;

files.forEach(file => {
  if (file.endsWith('.json') && file !== 'schema.json') {
    const configFile = path.join(configDir, file);
    const config = JSON.parse(fs.readFileSync(configFile, 'utf8'));
    const mergedConfig = mergeConfig(config);

    const valid = validate(mergedConfig);
    if (!valid) {
      console.error(`Validation errors in ${configFile}:`);
      console.error(validate.errors);
      hasErrors = true;
    } else {
      console.log(`${configFile} is valid.`);
    }
  }
});

if (hasErrors) {
  process.exit(1);
}
