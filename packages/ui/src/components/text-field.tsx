import type * as React from "react";
import { useId } from "react";
import {
  Input as InputPrimitive,
  Label as LabelPrimitive,
  TextField as TextFieldPrimitive,
  type TextFieldProps,
} from "react-aria-components";
import { cn } from "@/lib/utils";

export interface TextFieldPropsExt extends TextFieldProps {
  label?: string;
  error?: string;
  children?: React.ReactNode;
}

const TextField = ({
  label,
  error,
  className,
  children,
  ...props
}: TextFieldPropsExt) => {
  const errorId = useId();
  const describedBy = error ? errorId : undefined;
  return (
    <TextFieldPrimitive
      data-slot="text-field"
      className={cn("grid w-full gap-1.5", className)}
      {...props}
      isInvalid={Boolean(error) || props.isInvalid}
    >
      {label && (
        <LabelPrimitive className="text-sm font-medium text-foreground">
          {label}
        </LabelPrimitive>
      )}
      {children ?? (
        <InputPrimitive
          aria-describedby={describedBy}
          className="flex h-8 w-full rounded-[8px] border border-input bg-background px-3 py-1 text-[0.8125rem] shadow-sm transition-colors duration-150 file:border-0 file:bg-transparent file:text-[0.8125rem] file:font-medium placeholder:text-muted-foreground focus-visible:ring-2 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50"
        />
      )}
      {error && (
        <div id={errorId} role="alert" className="text-sm text-destructive">
          {error}
        </div>
      )}
    </TextFieldPrimitive>
  );
};

export { TextField };
