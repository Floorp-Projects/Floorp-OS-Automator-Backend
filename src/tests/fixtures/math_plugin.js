globalThis.Sapphillon = {
    Package: {
        meta: {
            name: "math-plugin",
            version: "1.0.0",
            description: "A practical math plugin for integration testing",
            author_id: "com.sapphillon.test",
            package_id: "com.sapphillon.test.math-plugin"
        },
        functions: {
            add: {
                description: "Adds two numbers",
                permissions: [],
                parameters: [
                    { idx: 0, name: "a", type: "number", description: "First number" },
                    { idx: 1, name: "b", type: "number", description: "Second number" }
                ],
                returns: [{
                    idx: 0,
                    type: "number",
                    description: "Sum"
                }],
                handler: (a, b) => {
                    console.log(`[JS] Adding ${a} + ${b}`);
                    return a + b;
                }
            },
            process_data: {
                description: "Process a data object",
                permissions: [],
                parameters: [
                    { idx: 0, name: "data", type: "object", description: "Data object with value and multiplier" }
                ],
                returns: [{
                    idx: 0,
                    type: "object",
                    description: "Processed result"
                }],
                handler: (data) => {
                    console.log(`[JS] Processing data: ${JSON.stringify(data)}`);
                    return {
                        original: data.value,
                        result: data.value * data.multiplier,
                        timestamp: new Date().toISOString()
                    };
                }
            }
        }
    }
};
