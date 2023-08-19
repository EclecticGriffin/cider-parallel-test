# pylint: disable=import-error
import calyx.builder as cb

MAX_CMDS = 15
ANS_MEM_LEN = 10


def insert_raise_err_if_i_eq_max_cmds(prog):
    """Inserts a the component `raise_err_if_i_eq_MAX_CMDS` into the program.

    It has:
    - one input, `i`.
    - one ref register, `err`.

    If `i` equals MAX_CMDS, it raises the `err` flag.
    """
    raise_err_if_i_eq_max_cmds: cb.ComponentBuilder = prog.component(
        "raise_err_if_i_eq_MAX_CMDS"
    )
    i = raise_err_if_i_eq_max_cmds.input("i", 32)
    err = raise_err_if_i_eq_max_cmds.reg("err", 1, is_ref=True)

    i_eq_max_cmds = raise_err_if_i_eq_max_cmds.eq_use(i, MAX_CMDS, 32)
    raise_err = raise_err_if_i_eq_max_cmds.reg_store(err, 1, "raise_err")

    raise_err_if_i_eq_max_cmds.control += [
        cb.if_with(
            i_eq_max_cmds,
            raise_err,
        )
    ]

    return raise_err_if_i_eq_max_cmds


def insert_main(prog, queue):
    """Inserts the component `main` into the program.
    This will be used to `invoke` the component `queue` and feed it a list of commands.
    """
    main: cb.ComponentBuilder = prog.component("main")

    # The user-facing interface of the `main` component is:
    # - input 1: a list of commands
    #    where each command is a 2-bit unsigned integer, with the following format:
    #    `0`: pop
    #    `1`: peek
    #    `2`: push
    # - input 2: a list of values to push
    #    where each value is a 32-bit unsigned integer
    #    the value at `i` is pushed if the command at `i` is `2`.
    # - output: a list of answers, reflecting any pops or peeks from the queue.
    #
    # The user-facing interface of the `queue` component is:
    # - input `cmd`
    #    where each command is a 2-bit unsigned integer, with the following format:
    #    `0`: pop
    #    `1`: peek
    #    `2`: push
    # - input `value`
    #   which is a 32-bit unsigned integer. If `cmd` is `2`, push this value.
    # - one ref register, `ans`, into which the result of a pop or peek is written.
    # - one ref register, `err`, which is raised if an error occurs.

    commands = main.seq_mem_d1("commands", 2, MAX_CMDS, 32, is_external=True)
    values = main.seq_mem_d1("values", 32, MAX_CMDS, 32, is_external=True)
    ans_mem = main.seq_mem_d1("ans_mem", 32, 10, 32, is_external=True)

    # The two components we'll use:
    queue = main.cell("myqueue", queue)
    raise_err_if_i_eq_max_cmds = main.cell(
        "raise_err_if_i_eq_MAX_CMDS", insert_raise_err_if_i_eq_max_cmds(prog)
    )

    # We will use the `invoke` method to call the `queue` component.
    # The queue component takes two inputs by reference and one input directly.
    # The two `ref` inputs:
    err = main.reg("err", 1)  # A flag to indicate an error
    ans = main.reg("ans", 32)  # A memory to hold the answer of a pop

    # We will set up a while loop that runs over the command list, relaying
    # the commands to the `queue` component.
    # It will run until the `err` flag is raised by the `queue` component.

    i = main.reg("i", 32)  # The index of the command we're currently processing
    j = main.reg("j", 32)  # The index on the answer-list we'll write to
    cmd = main.reg("command", 2)  # The command we're currently processing
    value = main.reg("value", 32)  # The value we're currently processing

    incr_i = main.incr(i, 32)  # i++
    incr_j = main.incr(j, 32)  # j++
    err_eq_0 = main.eq_use(err.out, 0, 1)  # is `err` flag down?
    cmd_le_1 = main.le_use(cmd.out, 1, 2)  # cmd <= 1

    read_cmd = main.mem_read_seq_d1(commands, i.out, "read_cmd_phase1")
    write_cmd_to_reg = main.mem_write_seq_d1_to_reg(commands, cmd, "write_cmd_phase2")

    read_value = main.mem_read_seq_d1(values, i.out, "read_value")
    write_value_to_reg = main.mem_write_seq_d1_to_reg(
        values, value, "write_value_to_reg"
    )
    write_ans = main.mem_store_seq_d1(ans_mem, j.out, ans.out, "write_ans")

    main.control += [
        cb.while_with(
            err_eq_0,  # Run while the `err` flag is down
            [
                read_cmd,
                write_cmd_to_reg,
                # `cmd := commands[i]`
                read_value,
                write_value_to_reg,
                # `value := values[i]`
                cb.invoke(  # Invoke the queue.
                    queue,
                    in_cmd=cmd.out,
                    in_value=value.out,
                    ref_ans=ans,
                    ref_err=err,
                ),
                cb.if_with(  # If it was a pop or a peek, write ans to the answer list
                    cmd_le_1,
                    [  # AM: I'd like to have an additional check hereL
                        # if err flag comes back raised,
                        # we do not perform this write_ans or this incr_j
                        write_ans,
                        incr_j,
                    ],
                ),
                incr_i,  # Increment the command index
                cb.invoke(  # If i = MAX_CMDS, raise error flag
                    raise_err_if_i_eq_max_cmds, in_i=i.out, ref_err=err
                ),  # AM: hella hacky
            ],
        ),
    ]