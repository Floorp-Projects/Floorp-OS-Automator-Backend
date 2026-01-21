globalThis.Sapphillon = {
    Package: {
        meta: {
            name: "file-plugin",
            version: "1.0.0",
            description: "A plugin for testing filesystem permissions",
            author_id: "com.sapphillon.test",
            package_id: "com.sapphillon.test.file-plugin"
        },
        functions: {
            read_file: {
                description: "Reads a file (requires FilesystemRead permission)",
                permissions: [{
                    type: "FilesystemRead",
                    level: 1,
                    display_name: "Filesystem Read",
                    description: "Read access to filesystem",
                    resource: []
                }],
                parameters: [
                    { idx: 0, name: "path", type: "string", description: "File path to read" }
                ],
                returns: [{
                    idx: 0,
                    type: "string",
                    description: "File contents"
                }],
                handler: async (path) => {
                    const content = await Deno.readTextFile(path);
                    return content;
                }
            },
            simple_function: {
                description: "A simple function that requires no permissions",
                permissions: [],
                parameters: [
                    { idx: 0, name: "message", type: "string", description: "Message to echo" }
                ],
                returns: [{
                    idx: 0,
                    type: "string",
                    description: "Echoed message"
                }],
                handler: (message) => {
                    return `Echo: ${message}`;
                }
            }
        }
    }
};
