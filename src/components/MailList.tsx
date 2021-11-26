import {
  FC,
  useCallback,
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
} from "react";

import throttle from "lodash/fp/throttle";

import { invoke } from "@tauri-apps/api";

import ConsoleState from "states/ConsoleState";
import LoginState from "states/LoginState";

import useTrigger from "utils/useTrigger";

import Box from "@mui/material/Box";
import List from "@mui/material/List";
import ListSubheader from "@mui/material/ListSubheader";
import ListItem from "@mui/material/ListItem";
import ListItemButton from "@mui/material/ListItemButton";
import ListItemIcon from "@mui/material/ListItemIcon";
import ListItemText from "@mui/material/ListItemText";
import AppBar from "@mui/material/AppBar";
import Typography from "@mui/material/Typography";
import IconButton from "@mui/material/IconButton";
import ToolTip from "@mui/material/Tooltip";
import Tab from "@mui/material/Tab";
import TabContext from "@mui/lab/TabContext";
import TabList from "@mui/lab/TabList";
import TabPanel from "@mui/lab/TabPanel";
import CircularProgress from "@mui/material/CircularProgress";

import Email from "@mui/icons-material/Email";
import Article from "@mui/icons-material/Article";
import EmailOutlined from "@mui/icons-material/EmailOutlined";
import Logout from "@mui/icons-material/Logout";
import Refresh from "@mui/icons-material/Refresh";

interface MailInfo {
  index: number;
  bytes: number;
}

type MailPartData =
  | {
      Text: string;
    }
  | {
      Html: string;
    }
  | {
      Bin: number[];
    };

interface MailData {
  raw: string;
  subject: string;
  from: string;
  to: string;
  time: string;
  parts: MailPartData[];
}

interface MailCardProps {
  mail: MailData;
}

const MailCard: FC<MailCardProps> = ({ mail }) => {
  const [tab, setTab] = useState("html-0");
  const [htmls, setHtmls] = useState<string[]>([]);
  const [texts, setTexts] = useState<string[]>([]);
  const [bins, setBins] = useState<number[][]>([]);

  useEffect(() => {
    const htmls: string[] = [];
    const texts: string[] = [];
    const bins: number[][] = [];
    for (const part of mail.parts) {
      if ("Html" in part) {
        htmls.push(part.Html);
      } else if ("Text" in part) {
        texts.push(part.Text);
      } else if ("Bin" in part) {
        bins.push(part.Bin);
      }
    }
    setHtmls(htmls);
    setTexts(texts);
    setBins(bins);
  }, [mail]);

  return (
    <Box
      sx={{
        flexGrow: 1,
        width: "calc(100vw - 200px)",
        overflowY: "auto",

        display: "flex",
        flexDirection: "column",
      }}
    >
      <Box
        sx={{
          display: "flex",
          flexDirection: "column",
          padding: "20px",
          border: "1px solid rgba(0, 0, 0, 0.12)",
        }}
        className="enable-user-select"
      >
        <Typography variant="h6" component="div">
          {mail.subject}
        </Typography>
        <Typography variant="body2" component="div">
          &emsp;由：{mail.from}
        </Typography>
        <Typography variant="body2" component="div">
          发往：{mail.to}
        </Typography>
        <Typography variant="body2" component="div">
          时间：{mail.time}
        </Typography>
      </Box>
      <TabContext value={tab}>
        <Box sx={{ borderBottom: 1, borderColor: "divider" }}>
          <TabList
            onChange={(_, newTab: string) => {
              setTab(newTab);
            }}
            aria-label="邮件标签页"
          >
            {htmls.map((_, idx, arr) => (
              <Tab
                label={arr.length > 1 ? `HTML #${idx}` : "HTML"}
                value={`html-${idx}`}
                key={`html-${idx}`}
              />
            ))}
            {texts.map((_, idx, arr) => (
              <Tab
                label={arr.length > 1 ? `纯文本 #${idx}` : "纯文本"}
                value={`text-${idx}`}
                key={`text-${idx}`}
              />
            ))}
            {bins.map((_, idx, arr) => (
              <Tab
                label={arr.length > 1 ? `附件 #${idx}` : "附件"}
                value={`bin-${idx}`}
                key={`bin-${idx}`}
              />
            ))}
            <Tab label="未解析" value="raw" />
          </TabList>
        </Box>
        {htmls.map((html, idx) => (
          <TabPanel
            className="enable-user-select mail-tab"
            value={`html-${idx}`}
            key={`html-${idx}`}
            sx={{ flexGrow: 1, overflowX: "auto" }}
          >
            <div dangerouslySetInnerHTML={{ __html: html }}></div>
          </TabPanel>
        ))}
        {texts.map((text, idx) => (
          <TabPanel
            className="mail-tab"
            value={`text-${idx}`}
            key={`text-${idx}`}
            sx={{
              flexGrow: 1,
              overflowX: "auto",
              overflowWrap: "break-word",
              whiteSpace: "pre-line",

              userSelect: "text",
            }}
          >
            {text}
          </TabPanel>
        ))}
        {bins.map((bin, idx) => (
          <TabPanel
            value={`bin-${idx}`}
            key={`bin-${idx}`}
            sx={{ flexGrow: 1 }}
          >
            {bin.length} bytes
          </TabPanel>
        ))}
        <TabPanel
          value="raw"
          sx={{
            flexGrow: 1,
            fontFamily:
              "source-code-pro, Menlo, Monaco, Consolas, 'Courier New', monospace, 'Segoe UI'",
            overflowWrap: "break-word",
            overflowX: "auto",
            whiteSpace: "pre",

            fontSize: "0.8em",
            lineHeight: "1em",
            userSelect: "text",
          }}
        >
          {mail.raw}
        </TabPanel>
      </TabContext>
    </Box>
  );
};

const MailPlaceholder: FC = () => {
  return (
    <Box
      sx={{
        flexGrow: 1,
        display: "flex",
        flexDirection: "column",
        justifyContent: "center",
        alignItems: "center",

        color: "text.secondary",
      }}
    >
      <Article sx={{ color: "#eeeeee", fontSize: 150 }} />
      请在邮件列表中选择邮件
    </Box>
  );
};

const MailList: FC = () => {
  const { logInfo, logError } = ConsoleState.useContainer();
  const { addr, username, setLogin } = LoginState.useContainer();

  const [listTrigger, toggleList] = useTrigger();

  const [mailData, setMailData] = useState<MailData | null>(null);
  const [fetching, setFetching] = useState(true);
  const [mailInfos, setMailInfos] = useState<MailInfo[]>([]);

  useEffect(() => {
    (async () => {
      try {
        const [mailNum, maildropBytes, msg] = (await invoke("stat")) as [
          number,
          number,
          string
        ];
        logInfo("command", await invoke("stat_msg"));
        logInfo("response", `${mailNum} ${maildropBytes} ${msg}`);
      } catch (err) {
        logInfo("command", await invoke("stat_msg"));
        logError("response", err);
      }
    })();
  }, [logInfo, logError]);

  useEffect(() => {
    (async () => {
      try {
        const [scanListings, msg] = (await invoke("list")) as [
          [number, number][],
          string
        ];
        const scanListingsText = scanListings
          .map(([index, bytes]) => `${index} ${bytes}`)
          .join("\r\n");
        logInfo("command", await invoke("list_msg"));
        logInfo("response", `${msg}\r\n${scanListingsText}\r\n.\r\n`);
        setMailInfos(scanListings.map(([index, bytes]) => ({ index, bytes })));
      } catch (err) {
        logInfo("command", await invoke("list_msg"));
        logError("response", err);
      }
      setFetching(false);
    })();
  }, [listTrigger, setFetching, logInfo, logError]);

  return (
    <>
      <AppBar
        position="static"
        sx={{ flexDirection: "row", alignItems: "center", zIndex: 1100 }}
      >
        <Email sx={{ fontSize: 40, margin: "10px", marginLeft: "20px" }} />
        <Box
          sx={{
            flexGrow: 1,
            display: "flex",
            flexDirection: "column",
          }}
        >
          <Typography
            variant="h6"
            component="div"
            sx={{ lineHeight: "1em", marginBottom: "3px" }}
          >
            {addr}
          </Typography>
          <Typography
            variant="body1"
            component="div"
            sx={{ color: "#dddddd", lineHeight: "1em" }}
          >
            {username}
          </Typography>
        </Box>
        <ToolTip title="收取邮件" sx={{ mr: 1 }}>
          <IconButton
            aria-label="收取邮件"
            color="inherit"
            onClick={(_) => {
              toggleList();
            }}
          >
            <Refresh />
          </IconButton>
        </ToolTip>
        <ToolTip title="登出" sx={{ mr: 3 }}>
          <IconButton
            aria-label="登出"
            color="inherit"
            onClick={async (_) => {
              try {
                logInfo("command", await invoke("quit_msg"));
                logInfo("response", await invoke("quit"));
                setLogin(false);
              } catch (err) {
                logError("response", err);
              }
            }}
          >
            <Logout />
          </IconButton>
        </ToolTip>
      </AppBar>
      <Box
        sx={{
          height: "calc(100vh - 60px)",
          flexGrow: 1,

          display: "flex",
          flexDirection: "row",
        }}
      >
        <List
          sx={{
            width: "100%",
            maxWidth: 200,
            bgcolor: "background.paper",
            border: "1px solid rgba(0, 0, 0, 0.12)",

            overflowY: "scroll",
          }}
          dense
          aria-labelledby="nested-list-subheader"
          subheader={
            <ListSubheader
              component="div"
              id="nested-list-subheader"
              sx={{ lineHeight: "3em" }}
            >
              邮件列表
            </ListSubheader>
          }
        >
          {fetching ? (
            <ListItem sx={{ marginTop: "10px", justifyContent: "center" }}>
              <CircularProgress />
            </ListItem>
          ) : (
            mailInfos.map((mail) => (
              <ListItem disablePadding>
                <ListItemButton
                  onClick={async () => {
                    try {
                      const payload = {
                        id: mail.index,
                      };
                      logInfo("command", await invoke("retr_msg", payload));
                      const [newMailData, msg] = (await invoke(
                        "retr",
                        payload
                      )) as [MailData, string];
                      setMailData(newMailData);
                      logInfo("response", `${msg}\r\n${newMailData.raw}.\r\n`);
                    } catch (err) {
                      logError("response", err);
                    }
                  }}
                >
                  <ListItemIcon>
                    <EmailOutlined />
                  </ListItemIcon>
                  <ListItemText
                    primary={`邮件 #${mail.index}`}
                    secondary={`${mail.bytes} Bytes`}
                  />
                </ListItemButton>
              </ListItem>
            ))
          )}
        </List>
        {mailData == null ? <MailPlaceholder /> : <MailCard mail={mailData} />}
      </Box>
    </>
  );
};

export default MailList;
