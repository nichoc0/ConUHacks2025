[](https://medium.com/?source=---top_nav_layout_nav----------------------------------)

[ ]

[](https://medium.com/m/signin?operation=register&redirect=https%3A%2F%2Fmedium.com%2Fnew-story&source=---top_nav_layout_nav-----------------------new_post_topnav-----------)

[Write](https://medium.com/m/signin?operation=register&redirect=https%3A%2F%2Fmedium.com%2Fnew-story&source=---top_nav_layout_nav-----------------------new_post_topnav-----------)

[Sign in](https://medium.com/m/signin?operation=login&redirect=https%3A%2F%2Fhandmadesoftware.medium.com%2Fstreamlit-asyncio-and-mongodb-f85f77aea825&source=post_page---top_nav_layout_nav-----------------------global_nav-----------)

![](https://miro.medium.com/v2/resize:fit:1273/0*Bpl8kZtxhyVNPJiH)

# Streamlit, asyncio and MongoDB

## Enabling async MongoDB operations in Streamlit.

[![Handmade Software](https://miro.medium.com/v2/resize:fill:88:88/1*RaOqlskDCYp7fWsSchoCeQ.png)](https://handmadesoftware.medium.com/?source=post_page---byline--f85f77aea825--------------------------------)

[Handmade Software](https://handmadesoftware.medium.com/?source=post_page---byline--f85f77aea825--------------------------------)

·[Follow](https://medium.com/m/signin?actionUrl=https%3A%2F%2Fmedium.com%2F_%2Fsubscribe%2Fuser%2Fcdd9b67800ea&operation=register&redirect=https%3A%2F%2Fhandmadesoftware.medium.com%2Fstreamlit-asyncio-and-mongodb-f85f77aea825&user=Handmade+Software&userId=cdd9b67800ea&source=post_page-cdd9b67800ea--byline--f85f77aea825---------------------post_header-----------)

7 min read**·**

May 21, 2024

[](https://medium.com/m/signin?actionUrl=https%3A%2F%2Fmedium.com%2F_%2Fbookmark%2Fp%2Ff85f77aea825&operation=register&redirect=https%3A%2F%2Fhandmadesoftware.medium.com%2Fstreamlit-asyncio-and-mongodb-f85f77aea825&source=---header_actions--f85f77aea825---------------------bookmark_footer-----------)

[](https://medium.com/m/signin?actionUrl=https%3A%2F%2Fmedium.com%2Fplans%3Fdimension%3Dpost_audio_button%26postId%3Df85f77aea825&operation=register&redirect=https%3A%2F%2Fhandmadesoftware.medium.com%2Fstreamlit-asyncio-and-mongodb-f85f77aea825&source=---header_actions--f85f77aea825---------------------post_audio_button-----------)

Streamlit
 is a wonderful tool for building dashboards with its peculiar execution
 model, but using asyncio data sources with it can be a real pain. This
article is about how you correctly use those two technologies together.

## Streamlit and beanie: What are those?

**Streamlit**
 is a Python library designed to make it easy for developers to create
beautiful, interactive web applications quickly, all using only Python
code. It’s popular for data science projects, allowing for rapid
prototyping and sharing of results.

We
 started using it for developing expert systems and dashboards, for
which the library is perfect: quick prototyping, nice testing utils and
responsiveness, everything a backend-heavy project’s interface needs.

**Beanie**
 is an asynchronous Python Object-Document Mapper (ODM) for MongoDB. It
leverages the power of Pydantic for data validation and the async/await
syntax of modern Python. Beanie makes it straightforward to work with
MongoDB documents as Python objects while handling asynchronous
operations seamlessly.

## Problem

As
 amazing as Streamlit is, there is one major problem with it: it
entirely ignores asyncio. And don’t get me wrong, in my not very humble
opinion asyncio implementation in Python is extremely unfortunate and
bulky, and just not pythonic in so many ways. But well, it gives you the
 benefits of particularly swift input-output, which isn’t so easy to
achieve with other tools.

The
 funny part is, that Streamlit is based on asyncio and utilizes Tornado
with asyncio engine in the background, but the event loop is not exposed
 to the eventlets and therefore cannot be used to start the coroutines.
The frontend elements are rendered from top to the bottom with
protobuffed data and web sockets as transport: every eventlet has an
event stream in both directions and synchronized in real time. Isn’t
that the web 3.0???

Seemingly
 this entire edifice is just screaming for asyncio, but the framework
was created to be accessible for scientists first and let’s be honest,
asyncio as it is right now would confuse even a professional software
developer if they never touched that before. So the entire API for
creating interface elements kept synchronous. And well, with tornado’s
execution model, it’s not that much of a deal, in fact it’s how asyncio
should have been done in Python in the first place.

## Possible solutions

So let’s say you need to execute an asyncio routine fetching some MongoDB data, and it takes some significant time:

```
async def fetch_data():
    count =  await Product.find(fetch_links=True).count()
    if not count:
        await Product(name='test').save()
    return await Product.find(fetch_links=True).to_list()
```

**Solution 1: asyncio.run(), okay for simple cases**

An obvious approach would be to use the sync-async bridge API of asyncio and just execute the code where it’s needed. `asyncio.run()` executes the code in a new event loop every time the requests comes, and that’s alright for simple cases. Full example:

```
import asyncio

from beanie import Document, init_beanie
from motor.motor_asyncio import AsyncIOMotorClient
import streamlit as st


def get_client(event_loop=None):
    if event_loop:
        client = AsyncIOMotorClient(
            "mongodb://127.0.0.1:27017/?directConnection=true&serverSelectionTimeoutMS=2000&appName=mongosh+2.0.2",
            io_loop=event_loop,
        )
    else:
        client = AsyncIOMotorClient(
            "mongodb://127.0.0.1:27017/?directConnection=true&serverSelectionTimeoutMS=2000&appName=mongosh+2.0.2",
        )
    return client


class Product(Document):
    name: str


async def init_database(client):
    database = client.get_database(name='asyncio_streamlit_db')
    await init_beanie(database=database, document_models=[Product])


async def fetch_data():
    count = await Product.find(fetch_links=True).count()
    if not count:
        await Product(name='test').save()
    return await Product.find(fetch_links=True).to_list()


async def main():
    client = get_client()
    await init_database(client)
    products = await fetch_data()
    for product in products:
        st.write(product)


if __name__ == '__main__':
    asyncio.run(main())
```

Where it breaks:

* creating a separate connection for every web socket request can clearly become a luxury once your project scales
* `init_beanie` can also take some while if you have many models to take in account
* it
  breaks with concurrent operations: tornado is a multithreaded
  application, and multithreading doesn’t go so simple with asyncio.

**Solution 2: optimized database initialization, one client per session?**

So
 instead of creating a separate connection with the database and
initializing beanie in every request, we could kind of cache it.
Streamlit runs are isolated, so no context will be preserved between the
 runs. Solution: session state. So what if we cache our connection for
each session?

```
import asyncio

from beanie import Document, init_beanie
from motor.motor_asyncio import AsyncIOMotorClient
import streamlit as st


def get_client(event_loop=None):
    if event_loop:
        client = AsyncIOMotorClient(
            "mongodb://127.0.0.1:27017/?directConnection=true&serverSelectionTimeoutMS=2000&appName=mongosh+2.0.2",
            io_loop=event_loop,
        )
    else:
        client = AsyncIOMotorClient(
            "mongodb://127.0.0.1:27017/?directConnection=true&serverSelectionTimeoutMS=2000&appName=mongosh+2.0.2",
        )
    return client


class Product(Document):
    name: str


async def init_database(client):
    database = client.get_database(name='asyncio_streamlit_db')
    await init_beanie(database=database, document_models=[Product])

COUNT = 10

async def fetch_data():
    count = await Product.find(fetch_links=True).count()
    if count < COUNT:
        for i in range(COUNT):
            await Product(name='test').save()
    return await Product.find(fetch_links=True).limit(COUNT).to_list()


async def main():
    if not st.session_state.get('client'):
        st.session_state.client = get_client()
        await init_database(st.session_state.client)
    products = await fetch_data()
    for product in products:
        st.write(product)
    st.button("Quick rerun")

if __name__ == '__main__':
    asyncio.run(main())
```

For
 static data the solution is just fine, and it’s quicker than the
previous one. But once you add buttons, especially with a default
setting of quick reruns being on, code re-exution leads to:

```
File "/home/thorin/PycharmProjects/streamlit_asyncio/venv/lib/python3.12/site-packages/streamlit/runtime/scriptrunner/script_runner.py", line 600, in _run_script
    exec(code, module.__dict__)
File "/home/thorin/PycharmProjects/streamlit_asyncio/solution2.py", line 49, in <module>
    asyncio.run(main())
File "/home/thorin/.pyenv/versions/3.12.3/lib/python3.12/asyncio/runners.py", line 194, in run
    return runner.run(main)
           ^^^^^^^^^^^^^^^^
File "/home/thorin/.pyenv/versions/3.12.3/lib/python3.12/asyncio/runners.py", line 118, in run
    return self._loop.run_until_complete(task)
           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
File "/home/thorin/.pyenv/versions/3.12.3/lib/python3.12/asyncio/base_events.py", line 687, in run_until_complete
    return future.result()
           ^^^^^^^^^^^^^^^
File "/home/thorin/PycharmProjects/streamlit_asyncio/solution2.py", line 43, in main
    products = await fetch_data()
               ^^^^^^^^^^^^^^^^^^
File "/home/thorin/PycharmProjects/streamlit_asyncio/solution2.py", line 32, in fetch_data
    count = await Product.find(fetch_links=True).count()
                  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
File "/home/thorin/PycharmProjects/streamlit_asyncio/venv/lib/python3.12/site-packages/beanie/odm/interfaces/find.py", line 273, in find
    return cls.find_many(
           ^^^^^^^^^^^^^^
File "/home/thorin/PycharmProjects/streamlit_asyncio/venv/lib/python3.12/site-packages/beanie/odm/interfaces/find.py", line 197, in find_many
    args = cls._add_class_id_filter(args, with_children)
           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
File "/home/thorin/PycharmProjects/streamlit_asyncio/venv/lib/python3.12/site-packages/beanie/odm/interfaces/find.py", line 458, in _add_class_id_filter
    and cls._inheritance_inited
        ^^^^^^^^^^^^^^^^^^^^^^^
File "/home/thorin/PycharmProjects/streamlit_asyncio/venv/lib/python3.12/site-packages/pydantic/_internal/_model_construction.py", line 242, in __getattr__
    raise AttributeError(item)
```

Which
 is the cryptic error for beanie being not initialized. Well, that’s
logical, we cache the connection, but not the event loop, which is
created every single time with `asyncio.run()`. Should I cache the event loop too?

**Solution 3: cache the event loop**

```
import asyncio

from beanie import Document, init_beanie
from motor.motor_asyncio import AsyncIOMotorClient
import streamlit as st


def get_client(event_loop=None):
    if event_loop:
        client = AsyncIOMotorClient(
            "mongodb://127.0.0.1:27017/?directConnection=true&serverSelectionTimeoutMS=2000&appName=mongosh+2.0.2",
            io_loop=event_loop,
        )
    else:
        client = AsyncIOMotorClient(
            "mongodb://127.0.0.1:27017/?directConnection=true&serverSelectionTimeoutMS=2000&appName=mongosh+2.0.2",
        )
    return client


class Product(Document):
    name: str


async def init_database(client):
    database = client.get_database(name='asyncio_streamlit_db')
    await init_beanie(database=database, document_models=[Product])

COUNT = 10

async def fetch_data():
    count = await Product.find(fetch_links=True).count()
    if count < COUNT:
        for i in range(COUNT):
            await Product(name='test').save()
    return await Product.find(fetch_links=True).limit(COUNT).to_list()

def get_event_loop():
    return asyncio.new_event_loop()


if not st.session_state.get('event_loop'):
    st.session_state.event_loop = get_event_loop()

if not st.session_state.get('client'):
    st.session_state.client = get_client(event_loop=st.session_state.event_loop)

async def main():
    await init_database(st.session_state.client)
    products = await fetch_data()
    for product in products:
        st.write(product)
    st.button("Quick rerun")

if __name__ == '__main__':
    st.session_state.event_loop.run_until_complete(main())
```

What I’ve done is caching the event loop in the session state and pass it to the motor client. The `asyncio.run()` was replaced with

```
st.session_state.event_loop.run_until_complete(main())
```

Now
 every session has its own event loop. Unfortunately seemingly working
solution will break, once you’ll try to execute multiple actions in
parallel, you can easily achieve it by clicking the button rapidly
multiple times.

```
RuntimeError: This event loop is already running
Traceback:
File "/home/thorin/PycharmProjects/streamlit_asyncio/venv/lib/python3.12/site-packages/streamlit/runtime/scriptrunner/script_runner.py", line 600, in _run_script
    exec(code, module.__dict__)
File "/home/thorin/PycharmProjects/streamlit_asyncio/solution3.py", line 57, in <module>
    st.session_state.event_loop.run_until_complete(main())
File "/home/thorin/.pyenv/versions/3.12.3/lib/python3.12/asyncio/base_events.py", line 663, in run_until_complete
    self._check_running()
File "/home/thorin/.pyenv/versions/3.12.3/lib/python3.12/asyncio/base_events.py", line 622, in _check_running
    raise RuntimeError('This event loop is already running')
```

Where it breaks:

* we still have to call `init_beanie` every single time, which will slow down operations significantly
* Once the event loop is started with `run_until_complete` no other eventlet can use it. You can, of course, wait until the task is finished, but this is not the path of a true samurai.

**Solution 4: cache everything globally and execute in a separate thread (best so far)**

Besides the session state, Streamlit also has a function to cache resources globally, `st.cache_resource`.
 If I cache the event loop globally just like that, I will run into the
same problem as with the previous solution, but the collision is even
more probable: now all the sessions share the same event loop.

The
 recommended solution for a multithreading application to enable asyncio
 properly is to use a worker thread. The thread is started with event
loop’s `run_forever` and accepts asyncio tasks from other threads with `asyncio.run_corouting_threadsafe`, that sends the tasks to the worker thread and waits for the result.

For
 now, this solution works the best for me. This assures the rest of the
code remains synchronous and only invokes so-unwanted asyncio functions
when it’s needed.

Besides
 that, if you move your beanie code (models and init function) out of
the script directory, its state won’t be reset each rerun, leaving `init_beanie` the result persistent. Full solution:

Script itself and utils.py

```
import asyncio
from asyncio import run_coroutine_threadsafe
from threading import Thread
import os
import sys
from motor.motor_asyncio import AsyncIOMotorClient
import streamlit as st

sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from utils import Product, init_database


def get_client(event_loop=None):
    if event_loop:
        client = AsyncIOMotorClient(
            "mongodb://127.0.0.1:27017/?directConnection=true&serverSelectionTimeoutMS=2000&appName=mongosh+2.0.2",
            io_loop=event_loop,
        )
    else:
        client = AsyncIOMotorClient(
            "mongodb://127.0.0.1:27017/?directConnection=true&serverSelectionTimeoutMS=2000&appName=mongosh+2.0.2",
        )
    return client


COUNT = 10


async def fetch_data():
    count = await Product.find(fetch_links=True).count()
    if count < COUNT:
        for i in range(COUNT):
            await Product(name='test').save()
    await asyncio.sleep(5)
    return await Product.find(fetch_links=True).limit(COUNT).to_list()


def get_event_loop():
    return asyncio.new_event_loop()


@st.cache_resource(show_spinner=False)
def create_loop():
    loop = asyncio.new_event_loop()
    thread = Thread(target=loop.run_forever)
    thread.start()
    return loop, thread


st.session_state.event_loop, worker_thread = create_loop()


def run_async(coroutine):
    return run_coroutine_threadsafe(coroutine, st.session_state.event_loop).result()


@st.cache_resource(show_spinner=False)
def setup_database():
    if client := st.session_state.get("db_client"):
        return client
    client = get_client(event_loop=st.session_state.event_loop)
    run_async(init_database(client=client))
    return client


st.session_state.db_client = setup_database()


def main():
    products = run_async(fetch_data())
    for product in products:
        st.write(product)
    st.button("Quick rerun")


if __name__ == '__main__':
    main()
```

```
from beanie import init_beanie, Document


class Product(Document):
    name: str
async def init_database(client):
    database = client.get_database(name='asyncio_streamlit_db')
    await init_beanie(database=database, document_models=[Product])
```

Where it breaks:

* Streamlit
  doesn’t have a way to catch the application or session shutdown or any
  kind of shutdown, so joining the worker thread is not possible. The
  thread remains alive and will keep the 8501 busy. Streamlit will connect
  to the next available port, but well, it’s not really optimal. Not a
  problem for Docker, but still a potential source of problems.

## Final words

All the examples can be found here: [https://github.com/thorin-schiffer/streamlit_asyncio/tree/master](https://github.com/thorin-schiffer/streamlit_asyncio/tree/master)

Don’t
 make the mistake I made, trying to keep my entire Streamlit app async.
It will break on multiple levels, I’ve tried to reach out to the
Streamlit developers myself to encourage them to think of native asyncio
 support in some form, utility like `run_async`
 can be a part of the framework easily. This seems quite surprising to
me because usually the Streamlit devs are quite responsive to the
community needs and having an asyncio supercharged data source for data
science dashboards, for which it was created, doesn’t seem such an
unrealistic scenario.

In
 the current state, Streamlit doesn’t expect the code to be async, it
breaks on many levels, one of the most annoying ones are the decorators
like `cache_resource` or recently
added majestic fragments are not going to work with asyncio coroutines.
The same is unfortunately correct for the callbacks.

Please feel free to add your suggestions and comments, let’s find the best solution together! Ciao 🖖

Written by Thorin Schiffer.

[Streamlit](https://medium.com/tag/streamlit?source=post_page-----f85f77aea825--------------------------------)

[Data Science](https://medium.com/tag/data-science?source=post_page-----f85f77aea825--------------------------------)

[Mongodb](https://medium.com/tag/mongodb?source=post_page-----f85f77aea825--------------------------------)

[Python](https://medium.com/tag/python?source=post_page-----f85f77aea825--------------------------------)

[](https://medium.com/m/signin?actionUrl=https%3A%2F%2Fmedium.com%2F_%2Fbookmark%2Fp%2Ff85f77aea825&operation=register&redirect=https%3A%2F%2Fhandmadesoftware.medium.com%2Fstreamlit-asyncio-and-mongodb-f85f77aea825&source=---footer_actions--f85f77aea825---------------------bookmark_footer-----------)

[![Handmade Software](https://miro.medium.com/v2/resize:fill:96:96/1*RaOqlskDCYp7fWsSchoCeQ.png)](https://handmadesoftware.medium.com/?source=post_page---post_author_info--f85f77aea825--------------------------------)

[Written by Handmade Software](https://handmadesoftware.medium.com/?source=post_page---post_author_info--f85f77aea825--------------------------------)[47 Followers](https://handmadesoftware.medium.com/followers?source=post_page---post_author_info--f85f77aea825--------------------------------)

**·**[25 Following](https://handmadesoftware.medium.com/following?source=post_page---post_author_info--f85f77aea825--------------------------------)

AWS, Python, Cloud: hire us now [https://www.handmadesoftware.nl/](https://www.handmadesoftware.nl/)

## Responses (1)

[](https://policy.medium.com/medium-rules-30e5502c4eb4?source=post_page---post_responses--f85f77aea825--------------------------------)

[What are your thoughts?](https://medium.com/m/signin?operation=register&redirect=https%3A%2F%2Fhandmadesoftware.medium.com%2Fstreamlit-asyncio-and-mongodb-f85f77aea825&source=---post_responses--f85f77aea825---------------------respond_sidebar-----------)

[ ]

Also publish to my profile

[![Mathieu Voisin](https://miro.medium.com/v2/resize:fill:58:58/1*2p7fPKM_6H_ExthZH-BPZg.jpeg)](https://medium.com/@mathieu.vn?source=post_page---post_responses--f85f77aea825----0----------------------------)

[Mathieu Voisin](https://medium.com/@mathieu.vn?source=post_page---post_responses--f85f77aea825----0----------------------------)

[Oct 28, 2024](https://medium.com/@mathieu.vn/thank-you-so-much-for-your-article-saved-me-a-lot-of-headaches-cfb09cbec7da?source=post_page---post_responses--f85f77aea825----0----------------------------)

<pre class="sp"><div class="xt l"><div class="bf b bg z bk"><div class="jl">Thank you so much for your article. Saved me a lot of headaches!</div></div></div></pre>

## More from Handmade Software

![Testing async MongoDB AWS applications with pytest](https://miro.medium.com/v2/resize:fit:1235/0*azoqlkniMW4j3KTX)

[![Handmade Software](https://miro.medium.com/v2/resize:fill:36:36/1*RaOqlskDCYp7fWsSchoCeQ.png)](https://handmadesoftware.medium.com/?source=post_page---author_recirc--f85f77aea825----0---------------------12099cce_5430_4e98_b53a_57513a240615-------)

[Handmade Software](https://handmadesoftware.medium.com/?source=post_page---author_recirc--f85f77aea825----0---------------------12099cce_5430_4e98_b53a_57513a240615-------)

[Testing async MongoDB AWS applications with pytestPractical guide and snippets](https://handmadesoftware.medium.com/testing-async-mongodb-aws-applications-with-pytest-ca28da5fa0c6?source=post_page---author_recirc--f85f77aea825----0---------------------12099cce_5430_4e98_b53a_57513a240615-------)

Jun 7, 2024

[](https://handmadesoftware.medium.com/testing-async-mongodb-aws-applications-with-pytest-ca28da5fa0c6?source=post_page---author_recirc--f85f77aea825----0---------------------12099cce_5430_4e98_b53a_57513a240615-------)

[6](https://handmadesoftware.medium.com/testing-async-mongodb-aws-applications-with-pytest-ca28da5fa0c6?source=post_page---author_recirc--f85f77aea825----0---------------------12099cce_5430_4e98_b53a_57513a240615-------)

[](https://medium.com/m/signin?actionUrl=https%3A%2F%2Fmedium.com%2F_%2Fbookmark%2Fp%2Fca28da5fa0c6&operation=register&redirect=https%3A%2F%2Fhandmadesoftware.medium.com%2Ftesting-async-mongodb-aws-applications-with-pytest-ca28da5fa0c6&source=---author_recirc--f85f77aea825----0-----------------bookmark_preview----12099cce_5430_4e98_b53a_57513a240615-------)

![Streamlit deployment on AWS ECS with Pulumi on a custom domain](https://miro.medium.com/v2/resize:fit:1235/1*Y-4TWOxunUBqyX8R0ES05w.png)

[![Handmade Software](https://miro.medium.com/v2/resize:fill:36:36/1*RaOqlskDCYp7fWsSchoCeQ.png)](https://handmadesoftware.medium.com/?source=post_page---author_recirc--f85f77aea825----1---------------------12099cce_5430_4e98_b53a_57513a240615-------)

[Handmade Software](https://handmadesoftware.medium.com/?source=post_page---author_recirc--f85f77aea825----1---------------------12099cce_5430_4e98_b53a_57513a240615-------)

[Streamlit deployment on AWS ECS with Pulumi on a custom domainGuide and snippets](https://handmadesoftware.medium.com/streamlit-deployment-on-aws-ecs-with-pulumi-on-a-custom-domain-8ec98d9a1dc1?source=post_page---author_recirc--f85f77aea825----1---------------------12099cce_5430_4e98_b53a_57513a240615-------)

Jul 5, 2024

[](https://handmadesoftware.medium.com/streamlit-deployment-on-aws-ecs-with-pulumi-on-a-custom-domain-8ec98d9a1dc1?source=post_page---author_recirc--f85f77aea825----1---------------------12099cce_5430_4e98_b53a_57513a240615-------)

[1](https://handmadesoftware.medium.com/streamlit-deployment-on-aws-ecs-with-pulumi-on-a-custom-domain-8ec98d9a1dc1?source=post_page---author_recirc--f85f77aea825----1---------------------12099cce_5430_4e98_b53a_57513a240615-------)[1](https://handmadesoftware.medium.com/streamlit-deployment-on-aws-ecs-with-pulumi-on-a-custom-domain-8ec98d9a1dc1?source=post_page---author_recirc--f85f77aea825----1---------------------12099cce_5430_4e98_b53a_57513a240615-------)

[](https://medium.com/m/signin?actionUrl=https%3A%2F%2Fmedium.com%2F_%2Fbookmark%2Fp%2F8ec98d9a1dc1&operation=register&redirect=https%3A%2F%2Fhandmadesoftware.medium.com%2Fstreamlit-deployment-on-aws-ecs-with-pulumi-on-a-custom-domain-8ec98d9a1dc1&source=---author_recirc--f85f77aea825----1-----------------bookmark_preview----12099cce_5430_4e98_b53a_57513a240615-------)

![New Relic Dashboards as Code](https://miro.medium.com/v2/resize:fit:1235/0*VwZseEPB_AzeAVKE)

[![Handmade Software](https://miro.medium.com/v2/resize:fill:36:36/1*RaOqlskDCYp7fWsSchoCeQ.png)](https://handmadesoftware.medium.com/?source=post_page---author_recirc--f85f77aea825----2---------------------12099cce_5430_4e98_b53a_57513a240615-------)

[Handmade Software](https://handmadesoftware.medium.com/?source=post_page---author_recirc--f85f77aea825----2---------------------12099cce_5430_4e98_b53a_57513a240615-------)

[New Relic Dashboards as CodeStructlog, New Relic and NerdGraph API](https://handmadesoftware.medium.com/new-relic-dashboards-as-code-d261acc30b6c?source=post_page---author_recirc--f85f77aea825----2---------------------12099cce_5430_4e98_b53a_57513a240615-------)

Aug 23, 2023

[](https://handmadesoftware.medium.com/new-relic-dashboards-as-code-d261acc30b6c?source=post_page---author_recirc--f85f77aea825----2---------------------12099cce_5430_4e98_b53a_57513a240615-------)

[4](https://handmadesoftware.medium.com/new-relic-dashboards-as-code-d261acc30b6c?source=post_page---author_recirc--f85f77aea825----2---------------------12099cce_5430_4e98_b53a_57513a240615-------)

[](https://medium.com/m/signin?actionUrl=https%3A%2F%2Fmedium.com%2F_%2Fbookmark%2Fp%2Fd261acc30b6c&operation=register&redirect=https%3A%2F%2Fhandmadesoftware.medium.com%2Fnew-relic-dashboards-as-code-d261acc30b6c&source=---author_recirc--f85f77aea825----2-----------------bookmark_preview----12099cce_5430_4e98_b53a_57513a240615-------)

![CDK, Python and moto](https://miro.medium.com/v2/resize:fit:1235/1*g9CSS1Pjs3PRVFHSOH7nQw.png)

[![CodeX](https://miro.medium.com/v2/resize:fill:36:36/1*VqH0bOrfjeUkznphIC7KBg.png)](https://medium.com/codex?source=post_page---author_recirc--f85f77aea825----3---------------------12099cce_5430_4e98_b53a_57513a240615-------)

In

[CodeX](https://medium.com/codex?source=post_page---author_recirc--f85f77aea825----3---------------------12099cce_5430_4e98_b53a_57513a240615-------)

by

[Handmade Software](https://handmadesoftware.medium.com/?source=post_page---author_recirc--f85f77aea825----3---------------------12099cce_5430_4e98_b53a_57513a240615-------)

[CDK, Python and motoMocking the AWS infrastructure from CDK CloudFormation configs](https://handmadesoftware.medium.com/cdk-python-and-moto-5e8ddffb5779?source=post_page---author_recirc--f85f77aea825----3---------------------12099cce_5430_4e98_b53a_57513a240615-------)

Jun 17, 2022

[](https://handmadesoftware.medium.com/cdk-python-and-moto-5e8ddffb5779?source=post_page---author_recirc--f85f77aea825----3---------------------12099cce_5430_4e98_b53a_57513a240615-------)

[1](https://handmadesoftware.medium.com/cdk-python-and-moto-5e8ddffb5779?source=post_page---author_recirc--f85f77aea825----3---------------------12099cce_5430_4e98_b53a_57513a240615-------)

[](https://medium.com/m/signin?actionUrl=https%3A%2F%2Fmedium.com%2F_%2Fbookmark%2Fp%2F5e8ddffb5779&operation=register&redirect=https%3A%2F%2Fmedium.com%2Fcodex%2Fcdk-python-and-moto-5e8ddffb5779&source=---author_recirc--f85f77aea825----3-----------------bookmark_preview----12099cce_5430_4e98_b53a_57513a240615-------)

[See all from Handmade Software](https://handmadesoftware.medium.com/?source=post_page---author_recirc--f85f77aea825--------------------------------)

## Recommended from Medium

![Integrating DeepSeek into your Python Applications](https://miro.medium.com/v2/resize:fit:1235/0*Yd3Pmxh4jbVdtGmj)

[![AI Advances](https://miro.medium.com/v2/resize:fill:36:36/1*R8zEd59FDf0l8Re94ImV0Q.png)](https://ai.gopubby.com/?source=post_page---read_next_recirc--f85f77aea825----0---------------------1b544589_7469_4fdd_b970_bbdeacd2cf6d-------)

In

[AI Advances](https://ai.gopubby.com/?source=post_page---read_next_recirc--f85f77aea825----0---------------------1b544589_7469_4fdd_b970_bbdeacd2cf6d-------)

by

[Wei-Meng Lee](https://weimenglee.medium.com/?source=post_page---read_next_recirc--f85f77aea825----0---------------------1b544589_7469_4fdd_b970_bbdeacd2cf6d-------)

[Integrating DeepSeek into your Python ApplicationsLearn
 how to use the DeepSeek chat and reasoning models in your Python
applications using Ollama, Hugging Face, and the DeepSeek API](https://weimenglee.medium.com/integrating-deepseek-into-your-python-applications-118e9f5da50f?source=post_page---read_next_recirc--f85f77aea825----0---------------------1b544589_7469_4fdd_b970_bbdeacd2cf6d-------)

6d ago

[](https://weimenglee.medium.com/integrating-deepseek-into-your-python-applications-118e9f5da50f?source=post_page---read_next_recirc--f85f77aea825----0---------------------1b544589_7469_4fdd_b970_bbdeacd2cf6d-------)

[496](https://weimenglee.medium.com/integrating-deepseek-into-your-python-applications-118e9f5da50f?source=post_page---read_next_recirc--f85f77aea825----0---------------------1b544589_7469_4fdd_b970_bbdeacd2cf6d-------)[4](https://weimenglee.medium.com/integrating-deepseek-into-your-python-applications-118e9f5da50f?source=post_page---read_next_recirc--f85f77aea825----0---------------------1b544589_7469_4fdd_b970_bbdeacd2cf6d-------)

[](https://medium.com/m/signin?actionUrl=https%3A%2F%2Fmedium.com%2F_%2Fbookmark%2Fp%2F118e9f5da50f&operation=register&redirect=https%3A%2F%2Fai.gopubby.com%2Fintegrating-deepseek-into-your-python-applications-118e9f5da50f&source=---read_next_recirc--f85f77aea825----0-----------------bookmark_preview----1b544589_7469_4fdd_b970_bbdeacd2cf6d-------)

![Alembic with SQLModel and FastAPI](https://miro.medium.com/v2/resize:fit:1235/1*ZOpWQYQ29uktI3CJAn7Egw.png)

[![Rajendra Kumar Yadav, M.Sc (CS)](https://miro.medium.com/v2/resize:fill:36:36/1*68epNmYFOLgBIur9LosWcw.png)](https://blogs.rajendrakumaryadav.dev/?source=post_page---read_next_recirc--f85f77aea825----1---------------------1b544589_7469_4fdd_b970_bbdeacd2cf6d-------)

[Rajendra Kumar Yadav, M.Sc (CS)](https://blogs.rajendrakumaryadav.dev/?source=post_page---read_next_recirc--f85f77aea825----1---------------------1b544589_7469_4fdd_b970_bbdeacd2cf6d-------)

[Alembic with SQLModel and FastAPIEfficient Database Migrations and Schema Management for Modern FastAPI Applications](https://blogs.rajendrakumaryadav.dev/alembic-with-sqlmodel-and-fastapi-a302ec10e079?source=post_page---read_next_recirc--f85f77aea825----1---------------------1b544589_7469_4fdd_b970_bbdeacd2cf6d-------)

Sep 5, 2024

[](https://blogs.rajendrakumaryadav.dev/alembic-with-sqlmodel-and-fastapi-a302ec10e079?source=post_page---read_next_recirc--f85f77aea825----1---------------------1b544589_7469_4fdd_b970_bbdeacd2cf6d-------)

[4](https://blogs.rajendrakumaryadav.dev/alembic-with-sqlmodel-and-fastapi-a302ec10e079?source=post_page---read_next_recirc--f85f77aea825----1---------------------1b544589_7469_4fdd_b970_bbdeacd2cf6d-------)

[](https://medium.com/m/signin?actionUrl=https%3A%2F%2Fmedium.com%2F_%2Fbookmark%2Fp%2Fa302ec10e079&operation=register&redirect=https%3A%2F%2Fblogs.rajendrakumaryadav.dev%2Falembic-with-sqlmodel-and-fastapi-a302ec10e079&source=---read_next_recirc--f85f77aea825----1-----------------bookmark_preview----1b544589_7469_4fdd_b970_bbdeacd2cf6d-------)

## Lists

[![](https://miro.medium.com/v2/resize:fill:87:87/0*r4yjMpEmqzHCUvWC.jpg)![](https://miro.medium.com/v2/resize:fill:87:87/1*bv2KUVNLi2sFNjBTdoBmWw.png)![](https://miro.medium.com/v2/resize:fill:87:87/0*zsngbTOmFCy6sUCx.jpeg)Predictive Modeling w/ Python20 stories**·**1811 saves](https://medium.com/@ben.putney/list/predictive-modeling-w-python-e3668ea008e1?source=post_page---read_next_recirc--f85f77aea825--------------------------------)

[![](https://miro.medium.com/v2/da:true/resize:fill:87:87/0*gzCeWxDtGmD23QR5)![](https://miro.medium.com/v2/resize:fill:87:87/1*di4WDrnS1F6_p9GWnxvPmg.png)![](https://miro.medium.com/v2/resize:fill:87:87/1*PzJLbFrFtNkqPsxielO8zA.jpeg)Coding &amp; Development11 stories**·**989 saves](https://medium.com/@jscribes/list/coding-development-e360d380bb82?source=post_page---read_next_recirc--f85f77aea825--------------------------------)

[![Principal Component Analysis for ML](https://miro.medium.com/v2/resize:fill:87:87/1*swd_PY6vTCyPnsgBYoFZfA.png)![Time Series Analysis](https://miro.medium.com/v2/resize:fill:87:87/1*8sSAHftNwd_RNJ3k4VA0pA.png)![deep learning cheatsheet for beginner](https://miro.medium.com/v2/resize:fill:87:87/1*uNyD4yNMH-DnOel1wzxOOA.png)Practical Guides to Machine Learning10 stories**·**2183 saves](https://destingong.medium.com/list/practical-guides-to-machine-learning-a877c2a39884?source=post_page---read_next_recirc--f85f77aea825--------------------------------)

[![](https://miro.medium.com/v2/resize:fill:87:87/1*rex1OZ5_KcxK2QrsZr3Cgw.jpeg)![](https://miro.medium.com/v2/resize:fill:87:87/1*I2o9__q4g1dzbcH9XRqcRg.png)![](https://miro.medium.com/v2/resize:fill:87:87/0*F6q2BN7oddumBDGY.png)ChatGPT prompts 51 stories**·**2527 saves](https://medium.com/@nicholas.michael.janulewicz/list/chatgpt-prompts-b4c47b8e12ee?source=post_page---read_next_recirc--f85f77aea825--------------------------------)

![Full-stack RAG: FastAPI Backend (Part 1)](https://miro.medium.com/v2/resize:fit:1235/1*AokNaDmuSDMPgBo3XR8X3Q.png)

[![Joey O'Neill](https://miro.medium.com/v2/resize:fill:36:36/1*S7Hnx5EhajgwRu1HCIqoBw.jpeg)](https://medium.com/@o39joey?source=post_page---read_next_recirc--f85f77aea825----0---------------------1b544589_7469_4fdd_b970_bbdeacd2cf6d-------)

[Joey O&#39;Neill](https://medium.com/@o39joey?source=post_page---read_next_recirc--f85f77aea825----0---------------------1b544589_7469_4fdd_b970_bbdeacd2cf6d-------)

[Full-stack RAG: FastAPI Backend (Part 1)All of my articles are 100% free to read. Non-members can read for free by clicking this friend link for the article.](https://medium.com/@o39joey/full-stack-rag-fastapi-backend-part-1-eab28eb21392?source=post_page---read_next_recirc--f85f77aea825----0---------------------1b544589_7469_4fdd_b970_bbdeacd2cf6d-------)

5d ago

[](https://medium.com/@o39joey/full-stack-rag-fastapi-backend-part-1-eab28eb21392?source=post_page---read_next_recirc--f85f77aea825----0---------------------1b544589_7469_4fdd_b970_bbdeacd2cf6d-------)

[15](https://medium.com/@o39joey/full-stack-rag-fastapi-backend-part-1-eab28eb21392?source=post_page---read_next_recirc--f85f77aea825----0---------------------1b544589_7469_4fdd_b970_bbdeacd2cf6d-------)

[](https://medium.com/m/signin?actionUrl=https%3A%2F%2Fmedium.com%2F_%2Fbookmark%2Fp%2Feab28eb21392&operation=register&redirect=https%3A%2F%2Fmedium.com%2F%40o39joey%2Ffull-stack-rag-fastapi-backend-part-1-eab28eb21392&source=---read_next_recirc--f85f77aea825----0-----------------bookmark_preview----1b544589_7469_4fdd_b970_bbdeacd2cf6d-------)

![Building a Custom Chatbot : A Streamlit Guide to AI Conversations](https://miro.medium.com/v2/resize:fit:1235/1*t3ThWQYkpzzVXL0i6InYRQ.jpeg)

[![Bryan Antoine](https://miro.medium.com/v2/resize:fill:36:36/1*7NU3-pPLlRg9A9iXOFAPmA.png)](https://medium.com/@b.antoine.se?source=post_page---read_next_recirc--f85f77aea825----1---------------------1b544589_7469_4fdd_b970_bbdeacd2cf6d-------)

[Bryan Antoine](https://medium.com/@b.antoine.se?source=post_page---read_next_recirc--f85f77aea825----1---------------------1b544589_7469_4fdd_b970_bbdeacd2cf6d-------)

[Building a Custom Chatbot : A Streamlit Guide to AI ConversationsThis
 guide will walk you through building a powerful chatbot application
using Streamlit, Groq, and SQLAlchemy. We’ll equip your app with…](https://medium.com/@b.antoine.se/building-a-custom-chatbot-a-streamlit-guide-to-ai-conversations-4ef524f0ea3f?source=post_page---read_next_recirc--f85f77aea825----1---------------------1b544589_7469_4fdd_b970_bbdeacd2cf6d-------)

Sep 20, 2024

[](https://medium.com/@b.antoine.se/building-a-custom-chatbot-a-streamlit-guide-to-ai-conversations-4ef524f0ea3f?source=post_page---read_next_recirc--f85f77aea825----1---------------------1b544589_7469_4fdd_b970_bbdeacd2cf6d-------)

[14](https://medium.com/@b.antoine.se/building-a-custom-chatbot-a-streamlit-guide-to-ai-conversations-4ef524f0ea3f?source=post_page---read_next_recirc--f85f77aea825----1---------------------1b544589_7469_4fdd_b970_bbdeacd2cf6d-------)

[](https://medium.com/m/signin?actionUrl=https%3A%2F%2Fmedium.com%2F_%2Fbookmark%2Fp%2F4ef524f0ea3f&operation=register&redirect=https%3A%2F%2Fmedium.com%2F%40b.antoine.se%2Fbuilding-a-custom-chatbot-a-streamlit-guide-to-ai-conversations-4ef524f0ea3f&source=---read_next_recirc--f85f77aea825----1-----------------bookmark_preview----1b544589_7469_4fdd_b970_bbdeacd2cf6d-------)

![12 Best Python Libraries You Must Know in 2025](https://miro.medium.com/v2/resize:fit:1235/0*18CvhiUbBAZsmtgc)

[![Rahul Sharma](https://miro.medium.com/v2/resize:fill:36:36/1*-9vsRc_qZE-EczvCxs73EA.png)](https://medium.com/@rs4528090?source=post_page---read_next_recirc--f85f77aea825----2---------------------1b544589_7469_4fdd_b970_bbdeacd2cf6d-------)

[Rahul Sharma](https://medium.com/@rs4528090?source=post_page---read_next_recirc--f85f77aea825----2---------------------1b544589_7469_4fdd_b970_bbdeacd2cf6d-------)

[12 Best Python Libraries You Must Know in 2025Libraries that every developer should know](https://medium.com/@rs4528090/12-best-python-libraries-you-must-know-in-2025-dd80552072b1?source=post_page---read_next_recirc--f85f77aea825----2---------------------1b544589_7469_4fdd_b970_bbdeacd2cf6d-------)

6d ago

[](https://medium.com/@rs4528090/12-best-python-libraries-you-must-know-in-2025-dd80552072b1?source=post_page---read_next_recirc--f85f77aea825----2---------------------1b544589_7469_4fdd_b970_bbdeacd2cf6d-------)

[172](https://medium.com/@rs4528090/12-best-python-libraries-you-must-know-in-2025-dd80552072b1?source=post_page---read_next_recirc--f85f77aea825----2---------------------1b544589_7469_4fdd_b970_bbdeacd2cf6d-------)

[](https://medium.com/m/signin?actionUrl=https%3A%2F%2Fmedium.com%2F_%2Fbookmark%2Fp%2Fdd80552072b1&operation=register&redirect=https%3A%2F%2Fmedium.com%2F%40rs4528090%2F12-best-python-libraries-you-must-know-in-2025-dd80552072b1&source=---read_next_recirc--f85f77aea825----2-----------------bookmark_preview----1b544589_7469_4fdd_b970_bbdeacd2cf6d-------)

![Building a Retail AI Chatbot: FastAPI, LangChain, PostgreSQL, and Market Basket Analysis](https://miro.medium.com/v2/resize:fit:1235/0*lOuolrr62SjfNW-V)

[![DataDrivenInvestor](https://miro.medium.com/v2/resize:fill:36:36/1*2mBCfRUpdSYRuf9EKnhTDQ.png)](https://medium.datadriveninvestor.com/?source=post_page---read_next_recirc--f85f77aea825----3---------------------1b544589_7469_4fdd_b970_bbdeacd2cf6d-------)

In

[DataDrivenInvestor](https://medium.datadriveninvestor.com/?source=post_page---read_next_recirc--f85f77aea825----3---------------------1b544589_7469_4fdd_b970_bbdeacd2cf6d-------)

by

[Shenggang Li](https://medium.com/@datalev?source=post_page---read_next_recirc--f85f77aea825----3---------------------1b544589_7469_4fdd_b970_bbdeacd2cf6d-------)

[Building a Retail AI Chatbot: FastAPI, LangChain, PostgreSQL, and Market Basket AnalysisUsing Large Language Model and Machine Learning to Boost Customer Engagement](https://medium.com/@datalev/building-a-retail-ai-chatbot-fastapi-langchain-postgresql-and-market-basket-analysis-30f752d9f404?source=post_page---read_next_recirc--f85f77aea825----3---------------------1b544589_7469_4fdd_b970_bbdeacd2cf6d-------)

Aug 25, 2024

[](https://medium.com/@datalev/building-a-retail-ai-chatbot-fastapi-langchain-postgresql-and-market-basket-analysis-30f752d9f404?source=post_page---read_next_recirc--f85f77aea825----3---------------------1b544589_7469_4fdd_b970_bbdeacd2cf6d-------)

[209](https://medium.com/@datalev/building-a-retail-ai-chatbot-fastapi-langchain-postgresql-and-market-basket-analysis-30f752d9f404?source=post_page---read_next_recirc--f85f77aea825----3---------------------1b544589_7469_4fdd_b970_bbdeacd2cf6d-------)[2](https://medium.com/@datalev/building-a-retail-ai-chatbot-fastapi-langchain-postgresql-and-market-basket-analysis-30f752d9f404?source=post_page---read_next_recirc--f85f77aea825----3---------------------1b544589_7469_4fdd_b970_bbdeacd2cf6d-------)

[](https://medium.com/m/signin?actionUrl=https%3A%2F%2Fmedium.com%2F_%2Fbookmark%2Fp%2F30f752d9f404&operation=register&redirect=https%3A%2F%2Fmedium.datadriveninvestor.com%2Fbuilding-a-retail-ai-chatbot-fastapi-langchain-postgresql-and-market-basket-analysis-30f752d9f404&source=---read_next_recirc--f85f77aea825----3-----------------bookmark_preview----1b544589_7469_4fdd_b970_bbdeacd2cf6d-------)

[See more recommendations](https://medium.com/?source=post_page---read_next_recirc--f85f77aea825--------------------------------)

[Help](https://help.medium.com/hc/en-us?source=post_page-----f85f77aea825--------------------------------)

[Status](https://medium.statuspage.io/?source=post_page-----f85f77aea825--------------------------------)

[About](https://medium.com/about?autoplay=1&source=post_page-----f85f77aea825--------------------------------)

[Careers](https://medium.com/jobs-at-medium/work-at-medium-959d1a85284e?source=post_page-----f85f77aea825--------------------------------)

[Press](https://handmadesoftware.medium.com/pressinquiries@medium.com?source=post_page-----f85f77aea825--------------------------------)

[Blog](https://blog.medium.com/?source=post_page-----f85f77aea825--------------------------------)

[Privacy](https://policy.medium.com/medium-privacy-policy-f03bf92035c9?source=post_page-----f85f77aea825--------------------------------)

[Terms](https://policy.medium.com/medium-terms-of-service-9db0094a1e0f?source=post_page-----f85f77aea825--------------------------------)

[Text to speech](https://speechify.com/medium?source=post_page-----f85f77aea825--------------------------------)

[Teams](https://medium.com/business?source=post_page-----f85f77aea825--------------------------------)

<iframe height="1" width="1" data-darkreader-inline-border-top="" data-darkreader-inline-border-right="" data-darkreader-inline-border-bottom="" data-darkreader-inline-border-left=""></iframe>
