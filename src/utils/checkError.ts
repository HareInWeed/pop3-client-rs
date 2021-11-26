const checkError = (err: unknown, handler: (msg: string) => void) => {
  if ((err as any).msg != null && typeof (err as any).msg === "string") {
    handler((err as any).msg);
  } else {
    console.error(err);
  }
};

export default checkError;
