# Todo

- [x] Refactor database pool, shared pool stored in rocket state.
- [ ] Optimize average response time calculation
  - [x] Seperate table for keeping track of all sort of stats
  - [ ] Implement calculations and endpoints
- [x] Mutate monitor_pool state when a monitor is removed, edited, or added.
  - [x] Remove monitor
  - [x] Edit monitor
  - [x] Add monitor
    - [~] fix random id gets generated when creating a monitor in memory, when creating it in the database it's id will be overwritten.
- [x] Fix monitor list navigation with hx-boost
