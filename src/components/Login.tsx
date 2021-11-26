import { FC } from "react";
import { useForm, Controller } from "react-hook-form";

import { invoke } from "@tauri-apps/api";

import checkError from "utils/checkError";

import LoginState from "states/LoginState";
import ConsoleState from "states/ConsoleState";

import Avatar from "@mui/material/Avatar";
import Button from "@mui/material/Button";
import TextField from "@mui/material/TextField";
import FormControlLabel from "@mui/material/FormControlLabel";
import Checkbox from "@mui/material/Checkbox";
import Box from "@mui/material/Box";
import Email from "@mui/icons-material/Email";
import Typography from "@mui/material/Typography";
import Container from "@mui/material/Container";
import CircularProgress from "@mui/material/CircularProgress";

type LoginData = {
  addr: string;
  username: string;
  password: string;
  withTls: boolean;
};

const Login: FC = () => {
  const { logInfo, logError } = ConsoleState.useContainer();
  const { setLogin, setAddr, setUsername } = LoginState.useContainer();

  const {
    handleSubmit,
    setError,
    control,
    formState: { isSubmitting },
  } = useForm<LoginData>();

  return (
    <Container
      component="main"
      maxWidth="xs"
      sx={{
        flexGrow: 1,
        display: "flex",
        flexDirection: "column",
        justifyContent: "center",
      }}
    >
      <Box
        sx={{
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
        }}
      >
        <Avatar sx={{ m: 1, bgcolor: "secondary.main", width: 60, height: 60 }}>
          <Email sx={{ fontSize: 40 }} />
        </Avatar>
        <Typography component="h1" variant="h5">
          POP3 客户端
        </Typography>
        <Box
          component="form"
          onSubmit={handleSubmit(async (data) => {
            // connect remote
            try {
              await invoke("connect", {
                addr: data.addr,
                withTls: data.withTls,
              });
              logInfo("network", `${data.addr} 成功连接`);
            } catch (err) {
              checkError(err, (message) =>
                setError("addr", { type: "network", message })
              );
              logError("network", err);
              return;
            }

            // user command
            try {
              const payload = {
                name: data.username,
              };
              logInfo("command", await invoke("user_msg", payload));
              logInfo("response", await invoke("user", payload));
            } catch (err) {
              checkError(err, (message) =>
                setError("username", { type: "network", message })
              );
              logError("response", err);
              return;
            }

            // user command
            try {
              const payload = {
                secret: data.password,
              };
              logInfo("command", `PASS ${"*".repeat(data.password.length)}`);
              logInfo("response", await invoke("pass", payload));
            } catch (err) {
              checkError(err, (message) =>
                setError("password", { type: "network", message })
              );
              logError("response", err);
              return;
            }

            setAddr(data.addr);
            setUsername(data.username);
            setLogin(true);
          })}
          noValidate
          sx={{ mt: 1 }}
        >
          <Controller
            name="addr"
            control={control}
            defaultValue=""
            rules={{ required: "请输入服务器地址" }}
            render={({
              field: { onChange, onBlur, value, ref },
              fieldState: { error },
              formState: { isSubmitting },
            }) => (
              <TextField
                id="addr"
                margin="normal"
                required
                fullWidth
                label="POP3 服务器地址"
                autoComplete="on"
                autoFocus
                error={!!error}
                helperText={error && error.message}
                disabled={isSubmitting}
                value={value}
                onChange={onChange}
                onBlur={onBlur}
                inputRef={ref}
              />
            )}
          />

          <Controller
            name="username"
            control={control}
            defaultValue=""
            rules={{ required: "请输入邮箱用户名" }}
            render={({
              field: { onChange, onBlur, value, ref },
              fieldState: { error },
              formState: { isSubmitting },
            }) => (
              <TextField
                id="username"
                margin="normal"
                required
                fullWidth
                label="邮箱用户名"
                autoComplete="username"
                error={!!error}
                helperText={error && error.message}
                disabled={isSubmitting}
                value={value}
                onChange={onChange}
                onBlur={onBlur}
                inputRef={ref}
              />
            )}
          />

          <Controller
            name="password"
            control={control}
            defaultValue=""
            rules={{ required: "请输入密码" }}
            render={({
              field: { onChange, onBlur, value, ref },
              fieldState: { error },
              formState: { isSubmitting },
            }) => (
              <TextField
                id="password"
                margin="normal"
                required
                fullWidth
                label="密码"
                type="password"
                autoComplete="current-password"
                error={!!error}
                helperText={error && error.message}
                disabled={isSubmitting}
                value={value}
                onChange={onChange}
                onBlur={onBlur}
                inputRef={ref}
              />
            )}
          />

          <Controller
            name="withTls"
            control={control}
            defaultValue={true}
            render={({ field: { onChange, onBlur, value, ref } }) => (
              <FormControlLabel
                control={
                  <Checkbox
                    value="tls"
                    color="primary"
                    disabled={isSubmitting}
                    checked={value}
                    onChange={onChange}
                    onBlur={onBlur}
                    inputRef={ref}
                  />
                }
                label="启用 TLS"
              />
            )}
          />

          <Button
            type="submit"
            fullWidth
            variant="contained"
            sx={{ mt: 3, mb: 2 }}
            disabled={isSubmitting}
          >
            {isSubmitting ? (
              <>
                登录中...&nbsp;
                <CircularProgress size={20} />
              </>
            ) : (
              "登录"
            )}
          </Button>
        </Box>
        <Typography
          variant="body2"
          color="text.secondary"
          align="center"
          sx={{ marginTop: "30px" }}
        >
          By HareInWeed
        </Typography>
      </Box>
    </Container>
  );
};

export default Login;
