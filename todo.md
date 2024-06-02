# Todo

- [x] Refactor database pool, currently it's opened and closed a lot. might get away with one shared pool stored in rocket state.
- [ ] Optimize average response time calculation, possibly by storing the sum of all response times and the count of requests.
  - Seperate table for keeping track of all sort of stats?
- [ ] Mutate monitor_pool state when a monitor is removed, edited, or added.
- [ ] Fix monitor list navigation with hx-boost
