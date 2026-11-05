export const retry = async <T>(fn: () => Promise<T>): Promise<T> => fn();
