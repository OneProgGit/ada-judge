#let lang = "en"
#let tr = (
  en: (
    page: "1 of 1",
  ),
  ru: (
    page: "Страница 1 из 1",
  )
)
#let t(key) = tr.at(lang).at(key)
#let template(title, subtitle: none, body) = {
  set page(
    paper: "a5",
    margin: (x: 20pt, top: 130pt, bottom: 50pt),
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
#let problem(title, contents) = {
  [
    = #title
    #contents $square.filled$
  ]
}

#template([Editorial for the Test Contest],
  [
    #problem("A. Sum of Numbers I",
      [
        The answer is $n + m$: it suffices to read the two numbers, add them and print the result. Under the constraint $(1 <= n, m <= 1000)$ the sum does not exceed $2000$ and fits in any integer type, so overflow is impossible. The running time is $O(1)$.
      ]
    )

    #pagebreak()

    #problem("B. Sum of Numbers II",
      [
        The solution is the same as in problem A. The subgroups differ only in their constraints: $1 <= n, m <= 500$ in the first one and up to $1000$ in the second. The algorithm does not change, so no separate code for the first subgroup is needed: a correct program passes both subgroups and receives 100 points.
      ]
    )

    #pagebreak()

    #problem("C. Sum of Numbers III",
      [
        The answer is $n + m$, as in problems A and B. Full marks for subgroup 1 are awarded for passing all of its tests, which include the sample tests. The problem has no other peculiarities.
      ]
    )

    #pagebreak()

    #problem("D. Sum of Numbers IV",
      [
        The algorithm is the same: the answer is $n + m$. The distinctive feature is the merged-subgroup scoring: for each subgroup the best result among all submissions is taken separately, and the points for the problem equal the sum of these maxima. Thus a submission that passes only subgroup 1 and another one that passes only subgroup 2 together bring the full 100 points. A correct solution passes all the tests of both subgroups and receives 100 points with a single submission.
      ]
    )

    #pagebreak()

    #problem("E. Sum of Numbers V",
      [
        The answer is $n + m$. The difference from problems A--D lies in the scoring system: the two tests besides the sample tests are scored independently, 50 points each, rather than as a single subgroup. This does not affect the solution: the problem admits no partial solution, it is either solved completely or not solved at all.
      ]
    )

    #pagebreak()

    #problem("F. Guess the Number",
      [
        Binary search applies. Keep the bounds of the interval of possible values $["lo", "hi"]$, initially $"lo" = 1$, $"hi" = t$. At each step make the query $x = floor(("lo" + "hi") / 2)$:
        - on the reply `=` the number has been guessed, and the program must terminate immediately;
        - on the reply `<` the sought number is less than $x$, and the interval shrinks to $["lo", x - 1]$;
        - on the reply `>` the interval shrinks to $[x + 1, "hi"]$.

        The length of the interval decreases by at least half with every query, so the number of queries does not exceed $ceil(log_2 t)$; for $t <= 10000$, $14$ queries suffice. The output buffer must be flushed after every query, and the program must terminate immediately after receiving the reply `=`.
      ]
    )

    #pagebreak()

    #problem("G. Add 1",
      [
        A run-twice problem is solved in parts. In the first run the program reads the number $n$ and prints $n + 1$. In the second run its input is the value printed by the first run: the value is transferred by the system, so no temporary files are needed. The program must print $v - 1$, that is, recover the original $n$.

        The main difficulty of the problem is technical rather than algorithmic: both runs must use only the standard input and output streams, print nothing extraneous and terminate correctly.
      ]
    )
  ]
)
