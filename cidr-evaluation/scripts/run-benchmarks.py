import csv
import subprocess
import time
import statistics as st
from collections import defaultdict
import pathlib

# Paths assumes you're running this script from the `futil` directory, i.e.
#   python3 evaluations/cidr-pldi-2022/process-data.py


def verify_interpreter_configuration():
    """
    Verifies the interpreter is in release mode and using
    the --no-verify flag.
    """

    def command_has_value(command, value, error):
        """
        Verifies that the stdout of this `command` has `value` in it.
        """
        process = subprocess.run(command, capture_output=True)
        assert value in str(process.stdout), error

    command_has_value(
        ["fud", "config", "stages.interpreter.exec"],
        "release",
        "The interpreter should be in release mode. "
        + "To fix this, run `fud config stages.interpreter.exec .<PATH-TO-CALYX>/target/release/cider`.",
    )

    command_has_value(
        ["fud", "config", "stages.interpreter.flags"],
        "--no-verify",
        "The interpreter should use the --no-verify flag. "
        + 'To fix this, run `fud config stages.interpreter.flags " --no-verify "`.',
    )


def get_csv_filename(name, lowered):
    """
    Uses the simulation name to produce the CSV file name, e.g. `Dot Product`
     -> `evaluations/cidr-pldi-2022/individual-results/Dot_Product.csv`

     We give slightly differe names to fully lowered Calyx programs, since Fud can't
     differentiate the two:
        `evaluations/cidr-pldi-2022/individual-results/Dot_Product-Lowered.csv`
    """
    return (
        "individual-results/"
        + name.replace(" ", "_")
        + ("-Lowered" if lowered else "")
        + ".csv"
    )


def process_data(dataset, is_fully_lowered, path, script, sim_num):
    """
    Runs the script for each iteration of dataset. `is_fully_lowered` is
    just used to distinguish file names.
    """
    for name, program in dataset:
        subprocess.run(
            [
                script,
                path + program,
                # Assumes that the data is the same path with `.data` appended.
                path + program + ".data",
                get_csv_filename(name, is_fully_lowered),
                str(sim_num),  # Number of simulations per program.
            ]
        )


def gather_data(dataset, is_fully_lowered):
    """
    Returns two mappings from simulation name to the data for both simulation
    and compilation times, e.g.
    {
      "Dot Product" : {"verilog": [1.1, 2.1], "interpreter": [1.9, 2.2], ...}
    }
    """
    simulations = {}
    compilations = {}
    for name, _ in dataset:
        # Just use the simulation name, e.g. Dot Product -> Dot_Product.csv
        with open(get_csv_filename(name, is_fully_lowered)) as file:
            # Mapping from stage to a list of durations.
            simtimes = defaultdict(list)
            comptimes = defaultdict(list)
            for row in csv.reader(file, delimiter=","):
                # e.g. icarus-verilog,simulate,0.126
                assert len(row) == 2, "expected CSV row: <stage-name>.<step>,<time>"
                stage_step, time = row
                stage, step = stage_step.split(".")
                time = float(time)
                if "compile" not in step:
                    # This is a simulation step.
                    simtimes[stage].append(time)
                else:
                    comptimes[stage].append(time)
            simulations[name] = simtimes
            compilations[name] = comptimes

    return simulations, compilations


def write_csv_results(type, results):
    """
    Writes a CSV file with the format:
    `type,stage,mean,median,stddev`

    to `evaluations/cidr-pldi-2022/statistics/<type>-results.csv`.
    """
    path = pathlib.Path(f"statistics/{type}-results.csv")
    preexisting = path.is_file()

    with open(path, "a", newline="") as file:
        writer = csv.writer(file, delimiter=",")
        if not preexisting:
            writer.writerow([type, "stage", "mean", "median", "stddev"])
        for name, data in results.items():
            for stage, times in data.items():
                mean = round(st.mean(times), 3)
                median = round(st.median(times), 3)
                stddev = round(st.stdev(times), 3)
                writer.writerow([name, stage, mean, median, stddev])


def write_to_file(data, filename):
    """
    Appends `data` to `filename`. Assumes that
    data is a list.
    """
    assert isinstance(data, list)
    with open(filename, "a") as file:
        file.writelines("\n".join(data))


def do_stats(data, is_fully_lowered: bool):
    # Process the CSV.
    simulations, compilations = gather_data(data, is_fully_lowered)
    # Provide meaning to the data.
    if is_fully_lowered:
        write_csv_results("simulation-fully-lowered", simulations)
    else:
        # No compilation for this, since we only run interpreter simulation for the fully-lowered script.
        write_csv_results("compilation", compilations)
        write_csv_results("simulation", simulations)


def run(data, script, sim_num=10):
    """
    Runs the simulation and data processing on the datasets.
    """
    # Run a different script for fully lowered Calyx. These are separated since Fud
    # has no way to dinstinguish profiling stage names based on previous stages.
    is_fully_lowered = "fully-lowered" in script

    # Run the bash script for each dataset.
    process_data(
        data,
        is_fully_lowered,
        path="benchmarks/",
        script=f"scripts/{script}",
        sim_num=sim_num,
    )

    do_stats(data, is_fully_lowered)


def setup():
    """Creates the necessary directories to store statistics."""
    subprocess.run(["mkdir", "-p", "individual-results"])
    subprocess.run(["mkdir", "-p", "statistics"])


if __name__ == "__main__":
    import sys

    setup()
    verify_interpreter_configuration()

    # A list of datasets to evaluate simulation performance, in the form:
    # (<table-name>, <program-path>). We just assume the data is at the same
    # path with `.data` appended. The path is relative to:
    #     futil/evaluations/cidr-pldi-2022/benchmarks
    datasets = [
        (
            "NTT 32",
            "ntt-32.futil",
        ),
        (
            "NTT 64",
            "ntt-64.futil",
        ),
        (
            "TCAM 32",
            "tcam-32.futil",
        ),
        (
            "TCAM 64",
            "tcam-64.futil",
        ),
        # Polybench
        (
            "Linear Algebra 2MM",
            "polybench/linear-algebra-2mm.fuse",
        ),
        (
            "Linear Algebra 3MM",
            "polybench/linear-algebra-3mm.fuse",
        ),
        (
            "Linear Algebra ATAX",
            "polybench/linear-algebra-atax.fuse",
        ),
        (
            "Linear Algebra BICG",
            "polybench/linear-algebra-bicg.fuse",
        ),
        (
            "Linear Algebra DOITGEN",
            "polybench/linear-algebra-doitgen.fuse",
        ),
        (
            "Linear Algebra DURBIN",
            "polybench/linear-algebra-durbin.fuse",
        ),
        (
            "Linear Algebra GEMM",
            "polybench/linear-algebra-gemm.fuse",
        ),
        (
            "Linear Algebra GEMVER",
            "polybench/linear-algebra-gemver.fuse",
        ),
        (
            "Linear Algebra GESUMMV",
            "polybench/linear-algebra-gesummv.fuse",
        ),
        (
            "Linear Algebra LU",
            "polybench/linear-algebra-lu.fuse",
        ),
        (
            "Linear Algebra LUDCMP",
            "polybench/linear-algebra-ludcmp.fuse",
        ),
        (
            "Linear Algebra MVT",
            "polybench/linear-algebra-mvt.fuse",
        ),
        (
            "Linear Algebra SYMM",
            "polybench/linear-algebra-symm.fuse",
        ),
        (
            "Linear Algebra SYR2K",
            "polybench/linear-algebra-syr2k.fuse",
        ),
        (
            "Linear Algebra SYRK",
            "polybench/linear-algebra-syrk.fuse",
        ),
        (
            "Linear Algebra TRISOLV",
            "polybench/linear-algebra-trisolv.fuse",
        ),
        (
            "Linear Algebra TRMM",
            "polybench/linear-algebra-trmm.fuse",
        ),
        (
            "Linear Algebra Cholesky",
            "polybench/linear-algebra-cholesky.fuse",
        ),
        (
            "Linear Algebra Gramschmidt",
            "polybench/linear-algebra-gramschmidt.fuse",
        ),
    ]

    lenet = [
        (
            "LeNet",
            "big/lenet.futil",
        )
    ]

    assert (
        len(sys.argv) == 2
    ), "Please provide exactly one benchmark set. Options are ('core', 'lenet', 'core-no-ntt64', 'full')"

    if sys.argv[1].lower() == "core":

        def program():
            print("Running the core benchmark suite...")
            # Run normal benchmarks on interpreter, Verilog, Icarus-Verilog.
            run(datasets, "evaluate.sh")
            # # Run benchmarks on fully lowered Calyx through the interpreter.
            run(datasets, "evaluate-fully-lowered.sh")

    elif sys.argv[1].lower() == "core-no-ntt64":
        datasets.remove(
            (
                "NTT 64",
                "ntt-64.futil",
            )
        )

        def program():
            print("Running the core benchmark suite without ntt-64...")
            # Run normal benchmarks on interpreter, Verilog, Icarus-Verilog.
            run(datasets, "evaluate.sh")
            # # Run benchmarks on fully lowered Calyx through the interpreter.
            run(datasets, "evaluate-fully-lowered.sh")

    elif sys.argv[1].lower() == "lenet":

        def program():
            print("Running lenet")
            run(lenet, "evaluate.sh", sim_num=5)

    elif sys.argv[1].lower() == "full":

        def program():
            print("Running the full benchmark suite...")
            # Run normal benchmarks on interpreter, Verilog, Icarus-Verilog.
            run(datasets, "evaluate.sh")
            # Run benchmarks on fully lowered Calyx through the interpreter.
            run(datasets, "evaluate-fully-lowered.sh")

            run(lenet, "evaluate.sh", sim_num=5)

    else:

        def program():
            print(
                "Not given a valid benchmark set, options are: ('core', 'lenet', 'core-no-ntt64', 'full')"
            )

    print("Beginning benchmarks...")
    begin = time.time()

    program()

    duration = (time.time() - begin) / 60.0
    print(f"Benchmarks took approximately: {int(duration)} minutes.")
