import { useState, useCallback } from "react";

const useTrigger = (): [boolean, () => void] => {
  const [trigger, setTrigger] = useState(false);
  const toggleTrigger = useCallback(() => {
    setTrigger((trigger) => !trigger);
  }, [setTrigger]);
  return [trigger, toggleTrigger];
};

export default useTrigger;
