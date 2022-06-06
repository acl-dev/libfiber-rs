use std::{mem, ptr};
use std::borrow::Borrow;
use std::os::raw::{c_int, c_uint, c_void};
use crate::libfiber::{ACL_FIBER, acl_fiber_create, acl_fiber_delay, acl_fiber_id, acl_fiber_kill, acl_fiber_killed, acl_fiber_running, acl_fiber_status, acl_fiber_switch, acl_fiber_yield, size_t};

pub struct Fiber {
    fiber: Option<*mut ACL_FIBER>,
    ///用户函数
    function: Box<dyn FnOnce(&Fiber, Option<*mut c_void>)>,
    ///用户参数
    param: Option<Box<*mut c_void>>,
}

impl Fiber {
    unsafe extern "C" fn fiber_main(_: *mut ACL_FIBER, arg: *mut c_void) {
        let mut fiber = arg as *mut Fiber;
        //fixme 这里如何调用闭包?
        let function = (*fiber).function.as_ref();
    }

    /// 创建纤程
    pub fn new<F>(function: F,
                  param: Option<*mut c_void>, size: size_t) -> Self
        where F: FnOnce(&Fiber, Option<*mut c_void>) + 'static
    {
        unsafe {
            let mut fiber = Fiber {
                fiber: None,
                function: Box::new(function),
                param: match param {
                    Some(param) => {
                        Some(Box::new(param))
                    }
                    None => None,
                },
            };
            let native_fiber = acl_fiber_create(Some(Fiber::fiber_main),
                                                &mut fiber as *mut _ as *mut c_void, size);
            fiber.fiber = Some(native_fiber);
            fiber
        }
    }

    ///主动让出CPU给其它纤程
    pub fn yields(&self) {
        unsafe {
            acl_fiber_yield();
        }
    }

    pub fn switch(&self) {
        unsafe {
            acl_fiber_switch();
        }
    }

    ///获取当前运行的纤程，如果没有正在运行的纤程将返回null
    pub fn current_running_fiber() -> *mut ACL_FIBER {
        unsafe {
            acl_fiber_running()
        }
    }

    ///获取指定纤程的id
    pub fn get_id(&self) -> c_uint {
        unsafe {
            match self.fiber {
                Some(fiber) => acl_fiber_id(fiber),
                None => 0,
            }
        }
    }

    ///获取指定纤程的状态
    pub fn get_status(&self) -> c_int {
        unsafe {
            match self.fiber {
                Some(fiber) => acl_fiber_status(fiber),
                None => 0,
            }
        }
    }

    ///纤程退出
    pub fn exit(&self) {
        unsafe {
            match self.fiber {
                Some(fiber) => acl_fiber_kill(fiber),
                None => {}
            }
        }
    }

    ///检查指定的纤程是否已经退出
    pub fn is_exited(&self) -> bool {
        unsafe {
            match self.fiber {
                Some(fiber) => acl_fiber_killed(fiber) > 0,
                None => true,
            }
        }
    }

    ///让当前纤程休眠一段时间
    pub fn delay(&self, milliseconds: c_uint) -> c_uint {
        unsafe {
            acl_fiber_delay(milliseconds)
        }
    }
}