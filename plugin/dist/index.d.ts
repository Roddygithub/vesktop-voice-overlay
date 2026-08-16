declare const plugin: {
    name: string;
    description: string;
    version: string;
    author: string;
    start: (api: any) => void;
    stop: () => void;
};

export { plugin as default, plugin };
