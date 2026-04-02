export const spawn = (..._args: unknown[]): never => {
  throw new Error("child_process is unavailable in service workers");
};

export default {
  spawn,
};
