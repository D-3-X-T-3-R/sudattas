const path = require("path");

module.exports = {
  webpack: {
    configure: (config) => {
      // Resolve 'admin' to the repo-root admin folder (no admin inside frontend/src)
      config.resolve.alias = {
        ...config.resolve.alias,
        admin: path.resolve(__dirname, "../admin"),
      };
      // Allow importing from outside src (ModuleScopePlugin only allows src by default)
      config.resolve.plugins = config.resolve.plugins.filter(
        (p) => p.constructor.name !== "ModuleScopePlugin"
      );
      // Transpile the root admin folder (JSX, etc.)
      const oneOfRule = config.module.rules.find((r) => r.oneOf);
      if (oneOfRule) {
        const jsRule = oneOfRule.oneOf.find(
          (r) => r.test && r.test.toString().includes("jsx")
        );
        if (jsRule && jsRule.include) {
          jsRule.include = [
            jsRule.include,
            path.resolve(__dirname, "../admin"),
          ];
        }
      }
      return config;
    },
  },
};
