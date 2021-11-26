import { useState } from "react";
import { createContainer } from "unstated-next";

const useLoginState = () => {
  const [login, setLogin] = useState(false);
  const [addr, setAddr] = useState("");
  const [username, setUsername] = useState("");
  return { login, addr, username, setLogin, setAddr, setUsername };
};

const LoginState = createContainer(useLoginState);

export default LoginState;
