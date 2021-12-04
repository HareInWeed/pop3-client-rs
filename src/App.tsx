import { FC } from "react";

import { createTheme, ThemeProvider } from "@mui/material/styles";

import CssBaseline from "@mui/material/CssBaseline";
import Box from "@mui/material/Box";
import Snackbar from "@mui/material/Snackbar";
import Alert from "@mui/material/Alert";
import Login from "components/Login";
import Console, { ConsoleMsg } from "components/Console";
import MailList from "components/MailList";

import SnackState from "states/SnackState";
import LoginState from "states/LoginState";
import ConsoleState from "states/ConsoleState";

const theme = createTheme({
  components: {
    MuiCssBaseline: {
      styleOverrides: `
        * {
          user-select: none;
        }
        .enable-user-select * {
          user-select: text;
        }
      `,
    },
  },
});

const App: FC = () => {
  return (
    <SnackState.Provider>
      <LoginState.Provider>
        <ConsoleState.Provider>
          <AppDisplay />
        </ConsoleState.Provider>
      </LoginState.Provider>
    </SnackState.Provider>
  );
};

const AppDisplay: FC = () => {
  const { login } = LoginState.useContainer();
  const { msg, severity, open, close } = SnackState.useContainer();
  const { consoleMsgs } = ConsoleState.useContainer();

  return (
    <ThemeProvider theme={theme}>
      <CssBaseline />
      <Box
        sx={{
          display: "flex",
          flexDirection: "column",
          alignItems: "stretch",
          height: "100vh",
        }}
      >
        {login ? <MailList /> : <Login />}
        <Console initialHeight={100}>
          {consoleMsgs.map((msg, idx) => (
            <ConsoleMsg msg={msg} key={idx} />
          ))}
        </Console>
        <Snackbar
          open={open}
          autoHideDuration={6000}
          onClose={() => {
            close();
          }}
        >
          <Alert
            onClose={() => {
              close();
            }}
            severity={severity}
            sx={{ width: "100%" }}
          >
            {msg}
          </Alert>
        </Snackbar>
      </Box>
    </ThemeProvider>
  );
};

export default App;
