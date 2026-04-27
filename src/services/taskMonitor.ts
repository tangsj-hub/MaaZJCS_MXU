/**
 * 任务监视器
 *
 * 任务状态追踪已迁移到 Rust 后端（通过 MaaFramework tasker sink 回调）。
 * 本模块保留 cancelTaskQueueMonitor，并新增循环任务的延迟迭代机制。
 */

const taskMonitorControllers = new Map<string, AbortController>();

/** 取消指定实例的任务队列监视器（如有） */
export function cancelTaskQueueMonitor(instanceId: string) {
  const controller = taskMonitorControllers.get(instanceId);
  if (!controller) {
    return;
  }

  controller.abort();
  taskMonitorControllers.delete(instanceId);
}

// ── 延迟循环任务机制 ─────────────────────────────────

/** 循环任务完成通知：Promise resolver 映射 */
const completionResolvers = new Map<string, () => void>();

/** 循环控制器映射 */
const loopControllers = new Map<string, AbortController>();

/**
 * 等待指定实例的任务批次完成。
 * 需要外部在收到 tasks-completed 事件时调用 notifyTasksCompleted。
 */
export function waitForTasksCompletion(instanceId: string, signal: AbortSignal): Promise<void> {
  return new Promise<void>((resolve, reject) => {
    if (signal.aborted) {
      reject(new Error('Aborted'));
      return;
    }
    completionResolvers.set(instanceId, resolve);
    const onAbort = () => {
      completionResolvers.delete(instanceId);
      reject(new Error('Aborted'));
    };
    signal.addEventListener('abort', onAbort, { once: true });
  });
}

/** 通知指定实例的任务批次已完成（由 App.tsx 的 state-changed 处理器调用） */
export function notifyTasksCompleted(instanceId: string) {
  const resolver = completionResolvers.get(instanceId);
  if (resolver) {
    completionResolvers.delete(instanceId);
    resolver();
  }
}

/** 可中断的 sleep */
function sleepAbortable(ms: number, signal: AbortSignal): Promise<void> {
  return new Promise((resolve, reject) => {
    if (signal.aborted) {
      reject(new Error('Aborted'));
      return;
    }
    const timer = setTimeout(resolve, ms);
    const onAbort = () => {
      clearTimeout(timer);
      reject(new Error('Aborted'));
    };
    signal.addEventListener('abort', onAbort, { once: true });
  });
}

/**
 * 注册循环任务控制器。
 * 在 Toolbar 启动带循环的任务批次时调用。
 */
export function registerLoopController(instanceId: string, controller: AbortController) {
  loopControllers.set(instanceId, controller);
}

/**
 * 清除所有延迟迭代（停止任务时调用）。
 */
export function clearDeferredIterations() {
  for (const controller of loopControllers.values()) {
    controller.abort();
  }
  loopControllers.clear();
  completionResolvers.clear();
}

/**
 * 清除指定实例的延迟迭代。
 */
export function clearInstanceLoop(instanceId: string) {
  const controller = loopControllers.get(instanceId);
  if (controller) {
    controller.abort();
    loopControllers.delete(instanceId);
  }
  completionResolvers.delete(instanceId);
}

export { sleepAbortable };
