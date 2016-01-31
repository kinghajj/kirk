(function() {var implementors = {};
implementors['crossbeam'] = ["impl&lt;'a, T&gt; <a class='trait' href='https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='crossbeam/mem/epoch/struct.Shared.html' title='crossbeam::mem::epoch::Shared'>Shared</a>&lt;'a, T&gt;",];implementors['libc'] = ["impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.stat.html' title='libc::stat'>stat</a>","impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.stat64.html' title='libc::stat64'>stat64</a>","impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.pthread_attr_t.html' title='libc::pthread_attr_t'>pthread_attr_t</a>","impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.sigset_t.html' title='libc::sigset_t'>sigset_t</a>","impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.sigaction.html' title='libc::sigaction'>sigaction</a>","impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.stack_t.html' title='libc::stack_t'>stack_t</a>","impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.siginfo_t.html' title='libc::siginfo_t'>siginfo_t</a>","impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.glob64_t.html' title='libc::glob64_t'>glob64_t</a>","impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.ucred.html' title='libc::ucred'>ucred</a>","impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.statfs.html' title='libc::statfs'>statfs</a>","impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.msghdr.html' title='libc::msghdr'>msghdr</a>","impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.termios.html' title='libc::termios'>termios</a>","impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.flock.html' title='libc::flock'>flock</a>","impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.ipc_perm.html' title='libc::ipc_perm'>ipc_perm</a>","impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.shmid_ds.html' title='libc::shmid_ds'>shmid_ds</a>","impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.dirent.html' title='libc::dirent'>dirent</a>","impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.dirent64.html' title='libc::dirent64'>dirent64</a>","impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.rlimit64.html' title='libc::rlimit64'>rlimit64</a>","impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.glob_t.html' title='libc::glob_t'>glob_t</a>","impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.ifaddrs.html' title='libc::ifaddrs'>ifaddrs</a>","impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.pthread_mutex_t.html' title='libc::pthread_mutex_t'>pthread_mutex_t</a>","impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.pthread_rwlock_t.html' title='libc::pthread_rwlock_t'>pthread_rwlock_t</a>","impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.pthread_mutexattr_t.html' title='libc::pthread_mutexattr_t'>pthread_mutexattr_t</a>","impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.pthread_cond_t.html' title='libc::pthread_cond_t'>pthread_cond_t</a>","impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.passwd.html' title='libc::passwd'>passwd</a>","impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.statvfs.html' title='libc::statvfs'>statvfs</a>","impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.sockaddr_nl.html' title='libc::sockaddr_nl'>sockaddr_nl</a>","impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.dqblk.html' title='libc::dqblk'>dqblk</a>","impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.signalfd_siginfo.html' title='libc::signalfd_siginfo'>signalfd_siginfo</a>","impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.fsid_t.html' title='libc::fsid_t'>fsid_t</a>","impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.mq_attr.html' title='libc::mq_attr'>mq_attr</a>","impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.cpu_set_t.html' title='libc::cpu_set_t'>cpu_set_t</a>","impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.sockaddr.html' title='libc::sockaddr'>sockaddr</a>","impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.sockaddr_in.html' title='libc::sockaddr_in'>sockaddr_in</a>","impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.sockaddr_in6.html' title='libc::sockaddr_in6'>sockaddr_in6</a>","impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.sockaddr_un.html' title='libc::sockaddr_un'>sockaddr_un</a>","impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.sockaddr_storage.html' title='libc::sockaddr_storage'>sockaddr_storage</a>","impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.addrinfo.html' title='libc::addrinfo'>addrinfo</a>","impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.sockaddr_ll.html' title='libc::sockaddr_ll'>sockaddr_ll</a>","impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.fd_set.html' title='libc::fd_set'>fd_set</a>","impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.tm.html' title='libc::tm'>tm</a>","impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.sched_param.html' title='libc::sched_param'>sched_param</a>","impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.Dl_info.html' title='libc::Dl_info'>Dl_info</a>","impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.epoll_event.html' title='libc::epoll_event'>epoll_event</a>","impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.utsname.html' title='libc::utsname'>utsname</a>","impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.utimbuf.html' title='libc::utimbuf'>utimbuf</a>","impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.timeval.html' title='libc::timeval'>timeval</a>","impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.timespec.html' title='libc::timespec'>timespec</a>","impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.rlimit.html' title='libc::rlimit'>rlimit</a>","impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.rusage.html' title='libc::rusage'>rusage</a>","impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.in_addr.html' title='libc::in_addr'>in_addr</a>","impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.in6_addr.html' title='libc::in6_addr'>in6_addr</a>","impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.ip_mreq.html' title='libc::ip_mreq'>ip_mreq</a>","impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.ipv6_mreq.html' title='libc::ipv6_mreq'>ipv6_mreq</a>","impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.hostent.html' title='libc::hostent'>hostent</a>","impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.iovec.html' title='libc::iovec'>iovec</a>","impl <a class='trait' href='core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='libc/struct.pollfd.html' title='libc::pollfd'>pollfd</a>",];implementors['log'] = ["impl <a class='trait' href='https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='enum' href='log/enum.LogLevel.html' title='log::LogLevel'>LogLevel</a>","impl <a class='trait' href='https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='enum' href='log/enum.LogLevelFilter.html' title='log::LogLevelFilter'>LogLevelFilter</a>","impl <a class='trait' href='https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='log/struct.LogLocation.html' title='log::LogLocation'>LogLocation</a>",];implementors['kirk'] = ["impl <a class='trait' href='https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='kirk/crew/channel/struct.Options.html' title='kirk::crew::channel::Options'>Options</a>","impl <a class='trait' href='https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html' title='core::marker::Copy'>Copy</a> for <a class='struct' href='kirk/crew/deque/struct.Options.html' title='kirk::crew::deque::Options'>Options</a>",];

            if (window.register_implementors) {
                window.register_implementors(implementors);
            } else {
                window.pending_implementors = implementors;
            }
        
})()
