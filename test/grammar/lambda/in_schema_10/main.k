schema Job:
    name: str

    createResource: () -> CronJob = lambda {
        lambda {
            CronJob {
                name = name
            }
        }()
    }
    getName: () -> str = lambda {
        lambda {name}()
    }

schema CronJob:
    name?: str

myJob = Job {
    name = "myJob"
}
myCronJob = myJob.createResource()
name = myJob.getName()
