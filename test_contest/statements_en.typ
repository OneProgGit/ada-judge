#let lang = "en"

#let tr = (
  en: (
    page: "1 of 1",
    tl: "Time limit",
    ml: "Memory limit",
    input: "Input format",
    output: "Output format",
    interactive_protocol: "Interactive protocol",
    first_run: "The first run",
    second_run: "The second run",
    samples: "Samples",
    comments: "Comments",
    scoring: "Scoring",
    subgroup: "Subgroup",
    limits: "Limits",
    score: "Score",
    depends_on: "Required subgroups",
  ),
  ru: (
    page: "Страница 1 из 1",
    tl: "Ограничение по времени",
    ml: "Ограничение по памяти",
    input: "Входные данные",
    output: "Выходные данные",
    interactive_protocol: "Протокол взаимодействия",
    first_run: "Первый запуск",
    second_run: "Второй запуск",
    samples: "Примеры",
    comments: "Примечание",
    scoring: "Система оценки",
    subgroup: "Подгруппа",
    limits: "Ограничения",
    score: "Баллы",
    depends_on: "Требуемые подгруппы",
  )
)

#let t(key) = tr.at(lang).at(key)

#let template(title, subtitle: none, body) = {
  set page(
    paper: "a5",
    margin: (x: 20pt, top: 100pt, bottom: 50pt),
    header: align(center)[
      #line(length: 100%)
      == #title
      #subtitle
      #line(length: 100%)
    ],
    footer: context align(center)[
      #line(length: 100%)
      #counter(page).display(t("page"), both: true)
    ],
  )

  set par(justify: true, linebreaks: "optimized")

  body
}

#let problem(title,
             tl,
             ml,
             statement,
             input: none,
             output: none,
             interactive_protocol: none,
             first_run: none,
             second_run: none,
             samples,
             comments: none,
             subgroups_scoring: none,
             per_test_scoring: none) = {
  [
    = #title
    #table(
      columns: (auto, auto),
      align: center,
      [
        === #t("tl")
      ],
      [
        #tl
      ],

      [
        === #t("ml")
      ],
      [
        #ml
      ],
    )
    #statement
    #if input != none {
      [
        == #t("input")
        #input
      ]
    }
    #if output != none {
      [
        == #t("output")
        #output
      ]
    }
    #if interactive_protocol != none {
      [
        == #t("interactive_protocol")
        #interactive_protocol
      ]
    }
    #if first_run != none {
      [
        == #t("first_run")
        #first_run
      ]
    }
    #if second_run != none {
      [
        == #t("second_run")
        #second_run
      ]
    }
    == #t("samples")
    #table(
      columns: (150pt, 150pt),
      table.header([stdin], [stdout]),
      align: start,
      ..samples.flatten(),
    )
    #if comments != none {
      [
        == #t("comments")
        #comments
      ]
    }
    #if subgroups_scoring != none {
      [
        == #t("scoring")
        #table(
          columns: (60pt, 100pt, 70pt, 150pt),
          table.header(t("subgroup"), t("limits"), t("score"), t("depends_on")),
          align: center,
          ..subgroups_scoring.flatten(),
        )
      ]
    }
    #if per_test_scoring != none {
      [
        == #t("scoring")
        #per_test_scoring
      ]
    }
  ]
}

#template([Test Contest], subtitle: [long, long ago],
  [
    #problem(
      "A. Sum of Numbers I",
      "1 second",
      "256 megabytes",
      [
        In the shop of the merchant Aristarkh Pankratyevich lie two purses of copper coins: the first holds $n$ coins, the second $m$. Being a prudent man, the merchant wishes to know the total sum in his possession without counting the coins one by one and being thereby distracted from his other trading affairs. You are given integers $n$ and $m$ -- print their sum.
      ],
      input:
      [
        The single line contains integers $n$ and $m$ $(1 <= n, m <= 1000)$.
      ],
      output:
      [
        Print a single number -- the answer to the problem.
      ],
      (
        [
          3 4
        ],
        [
          7
        ],
      ),
      subgroups_scoring:
      (
        [
          0
        ],
        [
          Тесты из условия
        ],
        [
          0
        ],
        [
          --
        ],
        [
          1
        ],
        [
          $(1 <= n, m <= 1000)$
        ],
        [
          100
        ],
        [
          0
        ]
      ),
    )

    #pagebreak()

    #problem(
      "B. Sum of Numbers II",
      "1 second",
      "256 megabytes",
      [
        The clerk Foma keeps two coffers of banknotes -- the first holds $n$ roubles, the second $m$. The shopkeeper demands that Foma report the total sum without delay, so that it may be entered into the ledger before evening prayers. You are given integers $n$ and $m$ -- print their sum.
      ],
      input:
      [
        The single line contains integers $n$ and $m$ $(1 <= n, m <= 1000)$.
      ],
      output:
      [
        Print a single number -- the answer to the problem.
      ],
      (
        [
          3 4
        ],
        [
          7
        ],
      ),
      subgroups_scoring:
      (
        [
          0
        ],
        [
          Тесты из условия
        ],
        [
          0
        ],
        [
          --
        ],
        [
          1
        ],
        [
          $(1 <= n, m <= 500)$
        ],
        [
          50
        ],
        [
          0
        ],
        [
          2
        ],
        [
          $(1 <= n, m <= 1000)$
        ],
        [
          50
        ],
        [
          --
        ]
      ),
    )

    #pagebreak()

    #problem(
      "C. Sum of Numbers III",
      "1 second",
      "256 megabytes",
      [
        The keeper of the post station keeps a tally of passing carriages: yesterday $n$ carriages arrived, and this morning $m$ more. How many carriages have passed the station since the keeper took up his post -- this is the question that has troubled him for three days running. You are given integers $n$ and $m$ -- print their sum.
      ],
      input:
      [
        The single line contains integers $n$ and $m$ $(1 <= n, m <= 1000)$.
      ],
      output:
      [
        Print a single number -- the answer to the problem.
      ],
      (
        [
          3 4
        ],
        [
          7
        ],
      ),
      subgroups_scoring:
      (
        [
          0
        ],
        [
          Тесты из условия
        ],
        [
          0
        ],
        [
          --
        ],
        [
          1
        ],
        [
          $(1 <= n, m <= 1000)$
        ],
        [
          50
        ],
        [
          179
        ],
      ),
    )

    #pagebreak()

    #problem(
      "D. Sum of Numbers IV",
      "1 second",
      "256 megabytes",
      [
        *This is a problem with subgroup merging.*\
        The same station keeper from the previous problem has, a week later, once again lost count of the passing carriages, and once again asks for your assistance -- the task being, at least, already familiar to him. You are given integers $n$ and $m$ -- print their sum.
      ],
      input:
      [
        The single line contains integers $n$ and $m$ $(1 <= n, m <= 1000)$.
      ],
      output:
      [
        Print a single number -- the answer to the problem.
      ],
      (
        [
          3 4
        ],
        [
          7
        ],
      ),
      subgroups_scoring:
      (
        [
          0
        ],
        [
          Тесты из условия
        ],
        [
          0
        ],
        [
          --
        ],
        [
          1
        ],
        [
          $(1 <= n, m <= 500)$
        ],
        [
          50
        ],
        [
          0
        ],
        [
          2
        ],
        [
          $(1 <= n, m <= 1000)$
        ],
        [
          50
        ],
        [
          --
        ]
      ),
    )

    #pagebreak()

    #problem(
      "E. Sum of Numbers V",
      "1 second",
      "256 megabytes",
      [
        The retired lieutenant Sinitsyn collects uniform buttons: one case holds $n$ buttons, the other $m$. Count the total number of buttons in the lieutenant's collection, that he may report it to his fellow officers at the club. You are given integers $n$ and $m$ -- print their sum.
      ],
      input:
      [
        The single line contains integers $n$ and $m$ $(1 <= n, m <= 1000)$.
      ],
      output:
      [
        Print a single number -- the answer to the problem.
      ],
      (
        [
          3 4
        ],
        [
          7
        ],
      ),
      per_test_scoring:
      (
        [
          *This problem has 2 tests in addition to the sample tests; each is independently worth 50 points.*
        ]
      ),
    )

    #pagebreak()

    #problem(
      "F. Guess the Number",
      "1 second",
      "256 megabytes",
      [
        *This is an interactive problem.*\
        The young cornet Sven Industrievich, for sport, has thought of a number between 1 and $t$, and challenges you to a duel of wits. You may make at most $k$ ($k$ being a hidden number) queries of the form `? x`:
        - If $x$ is the number thought of, Sven replies `=`, and you win -- no further queries are needed;
        - If the number thought of is less than $x$, Sven replies `<`;
        - Otherwise Sven replies `>`.
      ],
      interactive_protocol:
      [
        First, Sven gives you a natural number $t$ $(1 <= t <= 10000)$ -- the bound on the number thought of.\
        You may then make at most $t$ queries of the form `? x`, where $x$ is a natural number from 1 to $t$; to each query Sven replies with one of the symbols `<`, `>`, or `=`.\
        As soon as you receive the reply `=`, your program must terminate immediately.
      ],
      (
        [
          10\
          >\
          <\
          \=
        ],
        [
          \
          ? 5\
          ? 8\
          ? 7
        ]
      ),
      subgroups_scoring:
      (
        [
          0
        ],
        [
          Тесты из условия
        ],
        [
          0
        ],
        [
          --
        ],
        [
          1
        ],
        [
          $(1 <= t <= 10000)$
        ],
        [
          100
        ],
        [
          0
        ]
      ),
    )

    #pagebreak()

    #problem(
      "G. Add 1",
      "1 second",
      "256 megabytes",
      [
        *This is a run-twice problem.*\
        A clerk of the district office, having received a number $n$ from a petitioner, must add one to it and pass the paper on to the neighbouring department. The official there, upon receiving the paper bearing the number $n + 1$, must subtract one from it and return the number to the petitioner in its original form -- yet he neither knows nor is meant to know what the number $n$ was before the clerk touched it; only what was written on the paper he received.\
        Your program will be run twice: the first time it plays the part of the clerk, the second the part of the official. There is no connection between the two runs beyond what the first run prints to its output.
      ],
      first_run:
      [
        A single integer $n$ $(1 <= n <= 1000)$ is given. The program must print the number $n + 1$.
      ],
      second_run:
      [
        A single integer $v$ $(2 <= v <= 1001)$ is given -- the very number printed by the first run. The program must print the number $v - 1$, that is, the original number $n$.
      ],
      (
        [
          0\
          10
        ],
        [
          11
        ],
        [
          1\
          11
        ],
        [
          10
        ],
      ),
      comments:
      [
        The example shows both runs separately: first the initial run (input `10`, output `11`), then the second, which takes as input the result of the first (input `11`, output `10`).
      ],
      subgroups_scoring:
      (
        [
          0
        ],
        [
          Тесты из условия
        ],
        [
          0
        ],
        [
          --
        ],
        [
          1
        ],
        [
          $(1 <= n <= 1000)$
        ],
        [
          100
        ],
        [
          0
        ]
      ),
    )
  ]
)
