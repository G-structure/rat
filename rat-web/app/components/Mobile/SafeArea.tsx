import { JSX } from "solid-js";

interface SafeAreaProps {
  children: JSX.Element;
  top?: boolean;
  bottom?: boolean;
  left?: boolean;
  right?: boolean;
  all?: boolean;
  class?: string;
}

export function SafeArea(props: SafeAreaProps) {
  const classes = () => {
    const safeClasses: string[] = [];
    
    if (props.all) {
      safeClasses.push("safe-top", "safe", "safe-x");
    } else {
      if (props.top) safeClasses.push("safe-top");
      if (props.bottom) safeClasses.push("safe");
      if (props.left || props.right) safeClasses.push("safe-x");
    }
    
    return [props.class || "", ...safeClasses].join(" ").trim();
  };
  
  return (
    <div class={classes()}>
      {props.children}
    </div>
  );
}