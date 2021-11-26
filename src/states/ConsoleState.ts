import { useState, useCallback } from "react";
import { createContainer } from "unstated-next";

import checkError from "utils/checkError";

import SnackState from "states/SnackState";
import LoginState from "states/LoginState";

export interface ConsoleMsg {
  type: "network" | "command" | "response" | "response" | "other";
  level: "info" | "warning" | "error";
  msg: string;
}

const useConsoleState = () => {
  const { setLogin } = LoginState.useContainer();
  const { showMessage } = SnackState.useContainer();

  const [consoleMsgs, setConsoleMsg] = useState<ConsoleMsg[]>([]);
  const appendMsg = useCallback(
    (msg: Partial<ConsoleMsg>) => {
      setConsoleMsg((consoleMsg) => [
        ...consoleMsg,
        { type: "other", level: "info", msg: "", ...msg },
      ]);
    },
    [setConsoleMsg]
  );

  const logInfo = useCallback(
    (type: ConsoleMsg["type"], msg: string) => {
      appendMsg({
        type,
        level: "info",
        msg,
      });
    },
    [appendMsg]
  );

  const logError = useCallback(
    (type: ConsoleMsg["type"], err: unknown) => {
      if (typeof err === "string") {
        appendMsg({
          type,
          level: "error",
          msg: err,
        });
      } else {
        checkError(err, (msg) => {
          if (msg === "connection closed by remote") {
            appendMsg({
              type: "network",
              level: "error",
              msg,
            });
            setLogin(false);
            showMessage("服务器连接已断开，请重新登录", "error");
          } else {
            appendMsg({
              type,
              level: "error",
              msg,
            });
          }
        });
      }
    },
    [appendMsg, showMessage, setLogin]
  );

  return { consoleMsgs, appendMsg, logInfo, logError };
};

const ConsoleState = createContainer(useConsoleState);

export default ConsoleState;
