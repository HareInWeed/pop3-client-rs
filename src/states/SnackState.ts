import { useState, useCallback } from "react";
import { createContainer } from "unstated-next";

import { AlertProps } from "@mui/material/Alert";

const useSnackState = () => {
  const [open, setOpen] = useState(false);
  const [msg, setMessage] = useState("");
  const [severity, setSeverity] = useState<AlertProps["severity"]>("info");

  const close = useCallback(() => {
    setOpen(false);
  }, [setOpen]);
  const showMessage = useCallback(
    (msg: string, severity: AlertProps["severity"] = "info") => {
      setMessage(msg);
      setSeverity(severity);
      setOpen(true);
    },
    [setMessage, setSeverity]
  );

  return { open, msg, severity, close, showMessage };
};

const SnackState = createContainer(useSnackState);

export default SnackState;
