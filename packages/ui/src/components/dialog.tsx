import type * as React from "react";
import {
  Dialog as DialogPrimitive,
  type DialogProps,
  DialogTrigger as DialogTriggerPrimitive,
  ModalOverlay as ModalOverlayPrimitive,
  Modal as ModalPrimitive,
} from "react-aria-components";
import { cn } from "@/lib/utils";

const Dialog = ({ className, children, ...props }: DialogProps) => (
  <DialogPrimitive
    data-slot="dialog"
    className={cn(
      "relative flex w-full max-w-lg flex-col gap-4 border bg-background p-6 shadow-lg rounded-lg",
      className,
    )}
    {...props}
  >
    {children}
  </DialogPrimitive>
);

const DialogTrigger = DialogTriggerPrimitive;

const DialogContent = ({
  className,
  children,
  ...props
}: React.ComponentProps<typeof ModalPrimitive>) => (
  <ModalOverlayPrimitive
    isDismissable
    className="fixed inset-0 z-50 bg-black/80"
  >
    <ModalPrimitive
      data-slot="dialog-content"
      className={cn(
        "fixed left-[50%] top-[50%] z-50 grid w-full max-w-lg translate-x-[-50%] translate-y-[-50%] gap-4 border bg-background p-6 shadow-lg rounded-lg",
        className,
      )}
      {...props}
    >
      {children}
    </ModalPrimitive>
  </ModalOverlayPrimitive>
);

export { Dialog, DialogContent, DialogTrigger };
