import { FC, useState, useRef, useEffect, useCallback } from "react";

import throttle from "lodash/fp/throttle";

import Box from "@mui/material/Box";
import KeyboardArrowDown from "@mui/icons-material/KeyboardArrowDown";
import KeyboardArrowUp from "@mui/icons-material/KeyboardArrowUp";

import { ConsoleMsg as ConsoleMsgData } from "states/ConsoleState";

interface ConsoleMsgProps {
  msg: ConsoleMsgData;
}

export const ConsoleMsg: FC<ConsoleMsgProps> = ({ msg }) => {
  switch (msg.type) {
    case "network":
      if (msg.level === "error") {
        return (
          <p style={{ fontWeight: "bold", color: "#ff959e" }}>{msg.msg}</p>
        );
      } else {
        return (
          <p style={{ fontWeight: "bold", color: "#a0a0a0" }}>{msg.msg}</p>
        );
      }
    case "command":
      return (
        <p style={{ textIndent: "-1.6em", marginLeft: "1.6em" }}>
          <span style={{ color: "#E5C07B" }}>C: </span>
          {msg.msg}
        </p>
      );
    case "response":
      return (
        <p style={{ textIndent: "-1.6em", marginLeft: "1.6em" }}>
          <span style={{ color: "#58AFEF" }}>S: </span>
          {msg.level === "error" ? (
            <span style={{ fontWeight: "bold", color: "#FF616E" }}>+ERR </span>
          ) : (
            <span style={{ fontWeight: "bold", color: "#A5E075" }}>+OK </span>
          )}
          {msg.msg}
        </p>
      );
    case "other":
      return <p>{msg.msg}</p>;
    default:
      throw new Error("unexpected message type");
  }
};

interface DividerProps {
  onDrag?: (deltaHeight: number) => void;
  onToggle?: () => void;
  expand: boolean;
}

const Divider: FC<DividerProps> = ({ onDrag, onToggle, expand }) => {
  const [isDragging, setIsDragging] = useState<boolean>(false);
  const position = useRef<number>(0);
  const mouseMoveHandler = useCallback(
    throttle(1000 / 60, (event: MouseEvent) => {
      if (onDrag != null) {
        onDrag(-(event.pageY - position.current));
        position.current = event.pageY;
      }
    }),
    [onDrag]
  );
  const mouseUpHandler = useCallback(
    (_) => {
      document.removeEventListener("mousemove", mouseMoveHandler);
      document.removeEventListener("mouseup", mouseUpHandler);
      setIsDragging(false);
    },
    [setIsDragging, mouseMoveHandler]
  );

  useEffect(() => {
    if (onDrag != null && isDragging) {
      document.addEventListener("mousemove", mouseMoveHandler);
      document.addEventListener("mouseup", mouseUpHandler);
      return () => {
        document.removeEventListener("mousemove", mouseMoveHandler);
        document.removeEventListener("mouseup", mouseUpHandler);
      };
    }
  }, [onDrag, isDragging, mouseMoveHandler, mouseUpHandler]);

  return (
    <Box
      sx={{
        height: "30px",
        width: "100vw",

        display: "flex",
        flexDirection: "row",
      }}
    >
      <Box
        sx={{
          fontSize: "0.8em",
          padding: "5px 3px 5px 10px",
          borderRadius: "0 4px 0 0",

          backgroundColor: "#fff",

          boxShadow: "0px 0px 5px 5px rgba(0, 0, 0, .1)",
          cursor: "pointer",

          display: "flex",
          flexDirection: "row",
          alignItems: "center",
        }}
        onClick={(_) => {
          if (onToggle != null) {
            onToggle();
          }
        }}
      >
        <Box>交互过程</Box>
        {expand ? <KeyboardArrowUp /> : <KeyboardArrowDown />}
      </Box>
      <Box
        sx={{
          flexGrow: 1,
          height: "6px",
          alignSelf: "flex-end",
          backgroundColor: "#fff",
          borderTop: "2px solid #ccc",
          boxShadow: "0px -3px 5px rgba(0, 0, 0, .2)",
          cursor: "n-resize",
        }}
        onMouseDown={
          !expand
            ? (event) => {
                position.current = event.nativeEvent.pageY;
                setIsDragging(true);
              }
            : () => {}
        }
      ></Box>
    </Box>
  );
};

interface ConsoleProps {
  initialHeight: number;
}

const Console: FC<ConsoleProps> = ({ initialHeight = 100, children }) => {
  const bottomTag = useRef<HTMLSpanElement | null>(null);
  const [height, setHeight] = useState(initialHeight);
  const [close, setClose] = useState(false);

  useEffect(() => {
    if (bottomTag.current != null) {
      bottomTag.current.scrollIntoView({ behavior: "smooth" });
    }
  }, [children]);

  return (
    <Box
      sx={{
        display: "flex",
        flexDirection: "column",
        alignSelf: "stretch",
        zIndex: 1250,

        position: "fixed",
        bottom: 0,
        left: 0,
      }}
    >
      <Divider
        onDrag={
          close
            ? undefined
            : (deltaHeight) => {
                setHeight((height) => height + deltaHeight);
              }
        }
        onToggle={() => {
          setClose((close) => !close);
        }}
        expand={close}
      />
      <Box
        sx={{
          backgroundColor: "black",
          color: "white",
          fontFamily:
            "source-code-pro, Menlo, Monaco, Consolas, 'Courier New', monospace, 'Segoe UI'",
          whiteSpace: "pre",
          width: "100vw",
          overflowX: "auto",
          overflowY: "scroll",

          fontSize: "0.8em",
          lineHeight: "1em",

          height: close ? 0 : `${height}px`,
          padding: close ? 0 : "10px",
        }}
        className="enable-user-select"
      >
        {children}
        <span id="bottom_tag" ref={bottomTag}></span>
      </Box>
    </Box>
  );
};

export default Console;
