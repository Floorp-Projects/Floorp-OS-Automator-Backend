globalThis.Sapphillon = {
    Package: {
        meta: {
            name: "error-plugin",
            version: "1.0.0",
            description: "A plugin for testing error handling",
            author_id: "com.sapphillon.test",
            package_id: "com.sapphillon.test.error-plugin"
        },
        functions: {
            throw_immediate: {
                description: "Throws an immediate error",
                permissions: [],
                parameters: [],
                returns: [],
                handler: () => {
                    throw new Error("This is an immediate error");
                }
            },
            throw_async: {
                description: "Throws an async error",
                permissions: [],
                parameters: [],
                returns: [],
                handler: async () => {
                    await new Promise(resolve => setTimeout(resolve, 10));
                    throw new Error("This is an async error");
                }
            },
            async_success: {
                description: "Returns a value asynchronously",
                permissions: [],
                parameters: [
                    { idx: 0, name: "value", type: "string", description: "Value to transform" }
                ],
                returns: [{
                    idx: 0,
                    type: "string",
                    description: "Transformed value"
                }],
                handler: async (value) => {
                    await new Promise(resolve => setTimeout(resolve, 10));
                    return `async: ${value}`;
                }
            },
            return_null: {
                description: "Returns null",
                permissions: [],
                parameters: [],
                returns: [],
                handler: () => {
                    return null;
                }
            },
            no_op: {
                description: "Does nothing",
                permissions: [],
                parameters: [],
                returns: [],
                handler: () => {
                    // no-op
                }
            }
        }
    }
};
