import { reactive } from "vue";

export type DialogType = "info" | "success" | "warning" | "error";

export interface DialogState {
  isOpen: boolean;
  title: string;
  message: string;
  type: DialogType;
  isConfirm: boolean;
  confirmText: string;
  cancelText: string;
  resolve?: (value: boolean) => void;
}

export interface ToastItem {
  id: string;
  message: string;
  type: DialogType;
}

export const dialogState = reactive<DialogState>({
  isOpen: false,
  title: "",
  message: "",
  type: "info",
  isConfirm: false,
  confirmText: "确认",
  cancelText: "取消",
  resolve: undefined,
});

export const toastList = reactive<ToastItem[]>([]);

/**
 * 弹出赛博朋克统一风格的消息提示框 (替代原生的 alert)
 */
export function cyberAlert(
  message: string,
  title: string = "SYSTEM NOTICE // 系统提示",
  type: DialogType = "info"
): Promise<void> {
  return new Promise((resolve) => {
    dialogState.title = title;
    dialogState.message = message;
    dialogState.type = type;
    dialogState.isConfirm = false;
    dialogState.confirmText = "确认";
    dialogState.resolve = () => resolve();
    dialogState.isOpen = true;
  });
}

/**
 * 弹出赛博朋克统一风格的确认对话框 (替代原生的 confirm)
 */
export function cyberConfirm(
  message: string,
  title: string = "CONFIRM ACTION // 安全确认",
  type: DialogType = "warning"
): Promise<boolean> {
  return new Promise((resolve) => {
    dialogState.title = title;
    dialogState.message = message;
    dialogState.type = type;
    dialogState.isConfirm = true;
    dialogState.confirmText = "确认执行";
    dialogState.cancelText = "取消";
    dialogState.resolve = (result: boolean) => resolve(result);
    dialogState.isOpen = true;
  });
}

/**
 * 弹出赛博浮动轻提示 Toast
 */
export function cyberToast(
  message: string,
  type: DialogType = "info",
  duration: number = 2500
) {
  const id = Math.random().toString(36).substring(2, 9);
  toastList.push({ id, message, type });
  setTimeout(() => {
    const idx = toastList.findIndex((t) => t.id === id);
    if (idx !== -1) {
      toastList.splice(idx, 1);
    }
  }, duration);
}
