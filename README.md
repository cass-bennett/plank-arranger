# Plank Arranger
This program aims to solve the following problem:<br/>
A company buys planks of wood that come in a constant length, and cuts them
down into pieces.
If somebody at the company knows the lengths of all the pieces they'll
need, then what's the smallest number of planks they can use to make all
the pieces?

## Getting Started
To build this on your system, you'll first need to [install Rust and Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html), and [clone this repository](https://git-scm.com/book/en/v2/Git-Basics-Getting-a-Git-Repository)
to your local system.

Then you can compile it by going into the `plank-arranger` directory and
typing ```cargo build --release``` and then running it with
```./target/release/plank-arranger```

### Parameters
In order to run, the program needs to know how long the planks are that you'll
be cutting from, and it needs a file that contains a list of all the piece
lengths you have to make.
You can supply these as command-line arguments, like in
```./target/release/plank-arranger -p [LENGTH] -f [FILENAME]```
or you can run it without the arguments and the program will prompt you for
them.

## Explanation of the Algorithm
The program starts by sorting all the piece lengths from smallest to biggest.
Then it constructs a plank from the smallest n elements, with the highest
value of n as possible.
This n-element arrangement of pieces must have the smallest total out of
any n-element arrangement, because any other arrangement would have
equal or bigger sized pieces.
This means every plank in a valid solution will have the same number of
pieces as this first one does, or fewer.

From the above paragraph, we can determine a minimum number of planks in
the overall solution, and look for solutions with that number of planks.

With the first plank constructed, we can construct a second plank
having the same number of pieces as the first one, using the smallest pieces
not already in use by the first plank.
We continue like this until we either find a valid solution, or we have a total
length that's too big to fit on a plank.

When a total is too big then we can try decreasing the number of pieces on
the plank, but once we do that we'll make all subsequent planks have that
new number of pieces or fewer, so we'll only decrease it if we can do so
without adding more planks to the solution.

If a plank total is too big and we can't decrease the number of pieces, then
we go back to the last valid plank we had.
This plank should have started out having the smallest n-element subset of
the available pieces, so now we can
[change it to have the second smallest n-element subset](#finding-the-next-smallest-plank-arrangement),
and then make subsequent planks as before.

Once we've gone through all the valid n-element arrangements for a given plank,
then:
- We can see if it's possible to decrease the number of pieces on
this plank, without increasing the total number of planks
- If not, and if this plank is *not* the first one then we can eliminate it
and go to the most recent valid plank.
- If neither of the above apply then we've checked all the possibilities for
the current total number of planks, so we'll have to decrease the number of
pieces on this plank and also increase the total number of planks.

### Finding the Next-Smallest Plank Arrangement
Once we have a set of n pieces, we can find the set of n pieces with the
next-smallest total by looking at the differences between subsequent
available pieces.
If we switch out one piece for another, then the difference in the total
will be the difference between the two pieces.

For example, if we have:
```
            +-----+-----+-----+-----+-----+-----+
piece 	    |  1  |  2  |  4  |  7  |  11 |  14 |
            +-----+-----+-----+-----+-----+-----+
difference        1     2     3     4     3
            +-----+-----+-----+-----+-----+-----+
on plank?   |  1  |  1  |  1  |  0  |  0  |  0  |
            +-----+-----+-----+-----+-----+-----+
```
then the first change would have to be switching the 4 and 7, giving us:
```
            +-----+-----+-----+-----+-----+-----+
piece 	    |  1  |  2  |  4  |  7  |  11 |  14 |
            +-----+-----+-----+-----+-----+-----+
difference        1     2     3     4     3
            +-----+-----+-----+-----+-----+-----+
on plank?   |  1  |  1  |  0  |  1  |  0  |  0  |
            +-----+-----+-----+-----+-----+-----+
```
and changing the total from 7 to 10 (a difference of 3).

After that we can switch out 2 for 4 (adding 2 to the total and giving a state
of `101100`), or switch out 7 for 11 (adding 4 to the total and giving a state
of `110010`).
Obviously we'll use the former first because it results in a smaller total, but
we'll want to use the latter later on, so we handle this by having a queue
of [BitVec](https://docs.rs/bitvec/0.14.0/bitvec/vec/struct.BitVec.html)s,
prioritized by their corresponding totals.

### Accounting for Identical Pieces

In the naïve version of this "next plank" algorithm, having multiple identically
sized pieces will result in multiple identical plank arrangements.
For example, the set { 1, 2, 3, 3, 3, 3, 4 } with 3 pieces on the plank
would result in four arrangements of { 1, 2, 3 }, six arrangements of
{ 1, 3, 3 }, and so on.

To overcome this, when we add pieces to the plank we only use the rightmost
piece with the length we want, and when we remove them we only remove the
leftmost piece with the length we want.
So in the `BitVec` representing our plank state, all the `1`s for a given piece
length will be bunched up to the right.

Even with this modification, it's still possible to generate identical
arrangements; for example if our pieces are { 1, 2, 5, 5, 7, 7 } and we
have two pieces on a plank then our algorithm will do this:
```
               items in queue afterward
1 2 5 5 7 7    (state, total)
---------------------------------------
1 1 0 0 0 0  - (100100, 6)

1 0 0 1 0 0  - (010100, 7)
               (100001, 8)

0 1 0 1 0 0  - (100001, 8)
               (010001, 9)

1 0 0 0 0 1  - (010001, 9)
               (010001, 9)
```
So we end up with two instances of `010001` in our queue - one that came from
moving the rightmost `1` of `010100`, and one that came from moving the
leftmost `1` of `100001`.

That doesn't actually matter for this implementation though, because it uses a
[priority queue](https://docs.rs/priority-queue/0.6.0/priority_queue/struct.PriorityQueue.html#method.push)
that won't add `010001` twice, but just update the priority for the original
`010001`.